use std::path::PathBuf;

use aws_sdk_s3::Client;
use regex::Regex;
use rocket::State;
use crate::{error::Error, data::RssGenData};

pub fn is_xml(path: &PathBuf) -> bool {
    path.extension()
        .map(|s| s == "xml")
        .unwrap_or(false)
}

pub fn convert_simple_regex(input: &str) -> Result<Regex, Error> {
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

pub async fn get_site_text(url: &str) -> Result<String, Error> {
    let response = reqwest::get(url).await?;
    let text = response.text().await?;

    let text = text.replace("\n", "");
    let text = text.replace("\r", "");
    let text = text.replace("\t", "");

    Ok(text)
}

pub async fn get_site_text_dry(url: &str) -> Result<(), Error> {
    let response = reqwest::get(url).await?;
    let _ = response.text().await?;

    Ok(())
}

pub async fn get_gen_data(id_xml: PathBuf, client: &State<Client>) -> Result<RssGenData, Error> {
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
