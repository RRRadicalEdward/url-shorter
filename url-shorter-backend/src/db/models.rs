use crate::SHORT_URL_COUNT;
use mongodb::bson::{self, Document};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use url::Url as UrlParser;

#[derive(Deserialize, Serialize)]
pub struct Url<'u> {
    pub id: i64,
    pub url: &'u str,
    pub short_url: String,
}

impl<'u> Url<'u> {
    pub fn new(url: &'u str) -> anyhow::Result<Self> {
        UrlParser::parse(url)?;

        let unique_id = SHORT_URL_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        let short_url = radix_fmt::radix_36(unique_id).to_string();

        Ok(Url {
            id: unique_id as i64,
            short_url,
            url,
        })
    }
}

impl From<Url<'_>> for Document {
    fn from(source: Url<'_>) -> Self {
        bson::to_document(&source).expect("Convert URL to BSON should not panic")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_create_correct_url() {
        assert!(Url::new("http://hello-world.com/here").is_ok());
    }

    #[test]
    fn try_to_create_incorrect_url() {
        assert!(Url::new("htt/2131231231dfd").is_err());
    }
}
