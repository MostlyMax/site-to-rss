use std::path::PathBuf;

use aws_sdk_s3::Client;
use cached::proc_macro::cached;
use once_cell::sync::Lazy;
use regex::Regex;
use rocket::State;
use crate::{error::Error, data::RssGenData};

/// Utility function to check if the extenstion of a file is .xml
pub fn is_xml(path: &PathBuf) -> bool {
    path.extension()
        .map(|s| s == "xml")
        .unwrap_or(false)
}

/// Converts the simple templating into regex. Uses the following rules:
/// -       {*}      => .*?
/// -       {%}      => (.*?)
/// -       \        => \\ [theres something weird about escape sequeces when converting html body to string]
/// -       /        => \/ [same as above]
/// -       \n       => removed
/// -       \r       => removed
/// -       \t       => removed
/// -  >[white space] => ><
pub fn convert_simple_regex(input: &str) -> Result<Regex, Error> {
    let items_re = input.lines().collect::<String>();
    let items_re = items_re.replace("{%}", "(.*?)");
    let items_re = items_re.replace("{*}", ".*?");
    let items_re = items_re.replace(r"\", r"\\");
    let items_re = items_re.replace("/", r"\/");
    let items_re = items_re.replace("\n", "");
    let items_re = items_re.replace("\r", "");
    let items_re = items_re.replace("\t", "");

    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r">\s+<").unwrap());
    let items_re = RE.replace_all(&items_re, "><").to_string();

    let items_re = format!("(?ms){}", items_re);

    let Ok(re) = Regex::new(&items_re) else {
        return Err(Error::BadRequest("Invalid regex"));
    };

    Ok(re)
}

/// utility function to download html from site and format nicely
#[cached(size=32, result = true)]
pub async fn get_site_text(url: String) -> Result<String, Error> {
    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        return Err(Error::BadRequest("Bad url"));
    }
    let text = response.text().await?;

    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r">\s+<").unwrap());
    let text = RE.replace_all(&text, "><").to_string();
    let text = text.replace("\n", "");
    let text = text.replace("\r", "");
    let text = text.replace("\t", "");

    Ok(text)
}

/// does the same as get_site_text but without allocating and returning the string
#[cached(size=16, result = true)]
pub async fn get_site_text_dry(url: String) -> Result<(), Error> {
    let response = reqwest::get(url).await?;
    let _ = response.text().await?;

    Ok(())
}

/// downloads the s3 config object then generates the xml document from the site and regex rules
pub async fn get_gen_data(id_xml: PathBuf, client: &State<Client>) -> Result<RssGenData, Error> {
    if !is_xml(&id_xml) {
        return Err(Error::NotFound("Expected xml file"));
    }

    let Some(id) = id_xml.file_stem().and_then(|s| s.to_str()) else {
        return Err(Error::NotFound("File not found"));
    };

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

pub fn generate_preview(re: Regex, text: &String) -> Vec<(usize, String)> {
    let mut items_preview = Vec::new();

    if let Some(first_cap) = re.captures_iter(text).next() {
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

    items_preview
}
