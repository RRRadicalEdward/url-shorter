use mongodb::bson::{self, Document};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Url<'u> {
    pub id: u64,
    pub shorter_url: &'u str,
    pub url: &'u str,
}

impl From<Url<'_>> for Document {
    fn from(source: Url<'_>) -> Self {
        bson::to_document(&source).expect("Convert URL to BSON should not panic")
    }
}
