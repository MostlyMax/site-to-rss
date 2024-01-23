#[macro_use]
extern crate rocket;
use rocket::{form::Form, State};
use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::{Template, context};

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client;

use rss::{ChannelBuilder, ItemBuilder};

use std::path::PathBuf;

mod error;
mod data;
mod utils;
mod openai;

use error::Error;
use data::{FormWizGenerate, RssGenData};


#[get("/health")]
fn health() -> &'static str {
    ":3"
}

#[get("/")]
fn index() -> Template {
    Template::render("index", context!{ })
}

#[post("/generate-1", data = "<form>")]
async fn generate_1(form: Form<data::FormWiz0>) -> Result<Template, Template> {
    let Ok(text) = utils::get_site_text(&form.site_url).await else {
        return Err(Template::render("index", context! {
            site_url: form.site_url.clone(),
            error_msg: r#"<div class="form-item error"><p>
            Unable to download site! Check for any typos and make sure the link works!
            </p></div>"#
        }));
    };

    let text = text.replace("><", ">\n<");

    Ok(Template::render("form-wiz-1", context! {
        site_url: form.site_url.clone(),
        site_html: text
    }))
}

#[get("/autofill?<url>")]
async fn autofill(url: String) -> Option<String> {
    let Ok(text) = utils::get_site_text(&url).await else {
        return None
    };

    let text = text.replace("><", ">\n<");
    openai::autofill_test(&text).await
}

#[post("/generate-2", data = "<form>")]
async fn generate_2(form: Form<data::FormWiz1>) -> Result<Template, Template> {
    let text = utils::get_site_text(&form.site_url).await
        .expect("this should never fail if its the same url as used in generate_1");

    let Ok(re) = utils::convert_simple_regex(&form.items_regex) else {
        let text = text.replace("><", ">\n<");

        return Err(Template::render("form-wiz-1", context! {
            site_url: form.site_url.clone(),
            items_regex: form.items_regex.clone(),
            site_html: text,
            error_msg: r#"<div class="form-item error"><p>
            Something went wrong parsing your item filter. Ensure that there
            are no typos or extra brackets laying around!
            </p></div>"#
        }));
    };

    let mut items_preview = Vec::new();

    if let Some(first_cap) = re.captures_iter(&text).next() {
        const MAX_GROUPS: usize = 10;
        let mut current_group_no = 0;

        for group in first_cap.iter() {
            if current_group_no >= MAX_GROUPS { break }

            // 0th group contains full match
            if current_group_no <= 0 {
                current_group_no += 1;
                continue
            }

            items_preview.push((
                current_group_no,
                group
                    .and_then(|s| Some(s.as_str().to_string()))
                    .unwrap_or_default()
            ));

            current_group_no += 1;
        }
    };

    if items_preview.len() == 0 {
        let text = text.replace("><", ">\n<");

        return Err(Template::render("form-wiz-1", context! {
            site_url: form.site_url.clone(),
            items_regex: form.items_regex.clone(),
            site_html: text,
            error_msg: r#"<div class="form-item error"><p>
            Your item filter didn't find any matches. Ensure that there
            are no typos or extra brackets laying around and try again!
            </p></div>"#
        }));
    }

    Ok(Template::render("form-wiz-2", context! {
        site_url: form.site_url.clone(),
        items_regex: form.items_regex.clone(),
        items_preview: items_preview
    }))
}

#[post("/generate-3", data = "<form>")]
async fn generate_3(form: Form<data::FormWiz2>) -> Result<Template, Error> {
    Ok(Template::render("form-wiz-3", context! {
        site_url: form.site_url.clone(),
        items_regex: form.items_regex.clone(),
        item_title_no: form.item_title_no.clone(),
        item_url_no: form.item_url_no.clone(),
        item_content_no: form.item_content_no.clone(),
    }))
}


#[get("/<id_xml>")]
async fn get_rss(id_xml: PathBuf, client: &State<Client>) -> Result<String, Error> {
    let rss_gen_data = utils::get_gen_data(id_xml, client).await?;
    let text = utils::get_site_text(&rss_gen_data.site_url).await?;

    let mut items = Vec::new();
    for capture in rss_gen_data.items_regex.captures_iter(&text) {
        eprintln!("{:#?}", capture);
        let item_title = capture
            .get(rss_gen_data.item_title_no)
            .and_then(|s| Some(s.as_str().to_owned()));

        let item_url = rss_gen_data.item_url_no
            .and_then(|i| capture.get(i))
            .and_then(|s| Some(s.as_str().to_owned()));

        let item_content = rss_gen_data.item_content_no
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

/* Seperated /generate into an api call /api/generate and
   the normal one that currently generates an html template.
   this is in case i want to use client-side JS in the future instead
   of server-side rendering
*/
#[post("/generate", data = "<form>")]
#[allow(dead_code)]
async fn api_generate(form: Form<FormWizGenerate>, client: &State<Client>) -> Result<String, Error> {
    generate(form, client).await
}

#[post("/generate", data = "<form>")]
async fn template_generate(form: Form<FormWizGenerate>, client: &State<Client>) -> Result<Template, Error> {
    let id_xml = generate(form, client).await?;

    eprintln!("{}", id_xml);
    Ok(Template::render("generate", context! { id_xml: id_xml }))
}

async fn generate(form: Form<FormWizGenerate>, client: &State<Client>) -> Result<String, Error> {
    let _ = utils::get_site_text_dry(&form.site_url).await?;
    let rss_gen_data = RssGenData::try_from(form)?;

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
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = aws_sdk_s3::Client::new(&config);

    rocket::build()
        .mount("/static", FileServer::from(relative!("static")))
        .mount("/", routes![health, index, generate_1, generate_2, generate_3, template_generate])
        .mount("/rss/", routes![get_rss])
        .mount("/api/", routes![api_generate, autofill])
        .attach(Template::fairing())
        .manage(client)
}
