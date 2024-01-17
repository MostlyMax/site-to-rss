#[macro_use]
extern crate rocket;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use rocket::{form::Form, State};
use rocket::serde::{Serialize, Deserialize};
use rocket::fs::{FileServer, relative};
use rocket::serde::uuid::Uuid;

use regex::Regex;

use std::path::PathBuf;
use rss::{ChannelBuilder, ItemBuilder};

mod error;
use error::Error;


#[get("/health")]
fn index() -> &'static str {
    ":3"
}

#[derive(Debug, FromForm, UriDisplayQuery)]
struct RssFormData {
    site_url: String,
    global_regex: String,
    items_regex: String,
    feed_title: String,
    feed_url: Option<String>,
    feed_desc: Option<String>,
    item_title: usize,
    item_url: Option<usize>,
    item_content: Option<usize>,
}

impl RssFormData {
    fn get_site_url(&self) -> String {
        self.site_url.clone()
    }
}

impl TryFrom<Form<RssFormData>> for RssGenData {
    type Error = error::Error;

    fn try_from(value: Form<RssFormData>) -> Result<Self, error::Error> {
        let site_url = value.get_site_url();
        let items_regex = convert_simple_regex(&value.items_regex)?;
        let global_regex = convert_simple_regex(&value.global_regex)?;
        let id = Uuid::new_v4();

        Ok(RssGenData {
            id,
            site_url,
            items_regex,
            global_regex,

            feed_title: value.feed_title.clone(),
            feed_url: value.feed_url.clone(),
            feed_desc: value.feed_desc.clone(),
            item_title: value.item_title.clone(),
            item_url: value.item_url.clone(),
            item_content: value.item_content.clone(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RssGenData {
    id: Uuid,
    site_url: String,
    #[serde(with = "serde_regex")]
    global_regex: Regex,
    #[serde(with = "serde_regex")]
    items_regex: Regex,
    feed_title: String,
    feed_url: Option<String>,
    feed_desc: Option<String>,
    item_title: usize,
    item_url: Option<usize>,
    item_content: Option<usize>,
}

// #[derive(Debug)]
// struct PreviewRssData {
//     site_url: String,
//     // global: Regex,
//     items: Regex,
// }

fn convert_simple_regex(input: &str) -> Result<Regex, Error> {
    let items_re = input.lines().collect::<String>();
    let items_re = items_re.replace("{%}", "(.*?)");
    let items_re = items_re.replace("{*}", ".+?");
    let items_re = items_re.replace(r"\", r"\\");
    let items_re = items_re.replace("/", r"\/");
    let items_re = items_re.replace(">", r">\s*?");

    let items_re = format!("(?ms){}", items_re);

    let Ok(re) = Regex::new(&items_re) else {
        return Err(Error::BadRequest("Invalid regex".to_owned()));
    };

    Ok(re)
}

async fn get_site_text(url: &str) -> Result<String, Error> {
    let response = reqwest::get(url).await?;
    let text = response.text().await?;

    let text = text.replace("\n", "");
    let text = text.replace("\r", "");
    let text = text.replace("\t", "");

    Ok(text)
}

async fn get_site_text_dry(url: &str) -> Result<(), Error> {
    let response = reqwest::get(url).await?;
    let _ = response.text().await?;

    Ok(())
}

fn is_xml(path: &PathBuf) -> bool {
    path.extension()
        .map(|s| s == "xml")
        .unwrap_or(false)
}

async fn get_gen_data(id_xml: PathBuf, client: &State<Client>) -> Result<RssGenData, Error> {
    if !is_xml(&id_xml) {
        return Err(Error::NotFound("Expected xml file".to_owned()));
    }

    let Some(id) = id_xml.file_stem().and_then(|s| s.to_str()) else {
        return Err(Error::NotFound("File not found".to_owned()));
    };

    eprintln!("{}", id);

    let obj = client.get_object()
        .bucket("max-public-bucket")
        .key(id)
        .send()
        .await?;

    let raw_bytes = obj.body.collect().await?.into_bytes();
    let response  = std::str::from_utf8(&raw_bytes)?;
    let rss_gen_data = serde_json::from_str(response)?;

    Ok(rss_gen_data)
}

#[get("/rss/<id_xml>")]
async fn get_rss(id_xml: PathBuf, client: &State<Client>) -> Result<String, Error> {
    let rss_gen_data = get_gen_data(id_xml, client).await?;
    let text = get_site_text(&rss_gen_data.site_url).await?;

    // let text = rss_gen_data.global_regex.find(&text)
    //     .unwrap().as_str();

    // eprintln!("{}", text);

    let mut items = Vec::new();
    for capture in rss_gen_data.items_regex.captures_iter(&text) {
        eprintln!("{:#?}", capture);
        let item_title = capture
            .get(rss_gen_data.item_title)
            .and_then(|s| Some(s.as_str().to_owned()));

        let item_url = rss_gen_data.item_url
            .and_then(|i| capture.get(i))
            .and_then(|s| Some(s.as_str().to_owned()));

        let item_content = rss_gen_data.item_content
            .and_then(|i| capture.get(i))
            .and_then(|s| Some(s.as_str().to_owned()));

        items.push(ItemBuilder::default()
            .title(item_title)
            .link(item_url)
            .content(item_content)
            .build());
    }

    let channel = ChannelBuilder::default()
        .title(rss_gen_data.feed_title)
        .link(rss_gen_data.feed_url.unwrap_or_default())
        .description(rss_gen_data.feed_desc.unwrap_or_default())
        .items(items)
        .build()
        .to_string();

    Ok(channel)

}

// #[post("/preview", data = "<form>")]
// async fn preview(form: Form<RssFormData<'_>>) -> Result<String, BadRequest<String>> {
//     let site_url = form.get_site_url();
//     let items = convert_simple_regex(form.items)?;
//     let text = get_site_text(form.site_url).await?;

// }

#[post("/generate", data = "<form>")]
async fn generate(form: Form<RssFormData>, client: &State<Client>) -> Result<String, Error> {
    let _ = get_site_text_dry(&form.site_url).await?;
    let rss_gen_data = RssGenData::try_from(form)?;

    // push object to s3
    eprintln!("{:#?}", rss_gen_data);

    let serialized_data = serde_json::to_string(&rss_gen_data).unwrap();

    client.put_object()
        .bucket("max-public-bucket")
        .key(rss_gen_data.id)
        .body(serialized_data.into_bytes().into())
        .send()
        .await
        .unwrap();

    Ok(rss_gen_data.id.to_string())
}


#[launch]
async fn rocket() -> _ {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = aws_sdk_s3::Client::new(&config);

    rocket::build()
        .mount("/", FileServer::from(relative!("public")))
        .mount("/", routes![index, generate, get_rss])
        .manage(client)
}
