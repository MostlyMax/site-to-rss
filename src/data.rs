use regex::Regex;
use rocket::form::Form;
use rocket::serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::{error, utils};


#[derive(Debug, FromForm)]
pub struct FormWiz0 {
    pub site_url: String,
}

#[derive(Debug, FromForm)]
pub struct FormWiz1 {
    pub site_url: String,
    pub items_regex: String,
}

#[derive(Debug, FromForm)]
pub struct FormWiz2 {
    pub site_url: String,
    pub items_regex: String,
    pub item_title_no: usize,
    pub item_url_no: Option<usize>,
    pub item_content_no: Option<usize>,
}

#[derive(Debug, FromForm)]
pub struct FormWizGenerate {
    pub site_url: String,
    pub items_regex: String,
    pub item_title_no: usize,
    pub item_url_no: Option<usize>,
    pub item_content_no: Option<usize>,
    pub feed_title: String,
    pub feed_url: Option<String>,
    pub feed_desc: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RssGenData {
    pub id: Uuid,
    pub site_url: String,
    #[serde(with = "serde_regex")]
    pub items_regex: Regex,
    pub raw_items_regex: String,
    pub feed_title: String,
    pub feed_url: Option<String>,
    pub feed_desc: Option<String>,
    pub item_title_no: usize,
    pub item_url_no: Option<usize>,
    pub item_content_no: Option<usize>,
}

impl TryFrom<Form<FormWizGenerate>> for RssGenData {
    type Error = error::Error;

    fn try_from(value: Form<FormWizGenerate>) -> Result<Self, error::Error> {
        let items_regex = utils::convert_simple_regex(&value.items_regex)?;
        let id = Uuid::new_v4();

        Ok(RssGenData {
            id,
            items_regex,
            raw_items_regex: value.items_regex.clone(),
            site_url: value.site_url.clone(),
            feed_title: value.feed_title.clone(),
            feed_url: value.feed_url.clone(),
            feed_desc: value.feed_desc.clone(),
            item_title_no: value.item_title_no.clone(),
            item_url_no: value.item_url_no.clone(),
            item_content_no: value.item_content_no.clone(),
        })
    }
}
