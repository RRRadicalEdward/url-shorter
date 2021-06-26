use super::schema::urls;

#[derive(Queryable, Insertable)]
#[table_name = "urls"]
pub struct Url<'u> {
    pub id: i64,
    pub shorter_url: &'u str,
    pub url: &'u str,
}

impl<'u> Url<'u> {
    pub fn new(shorter_url: &'u str, url: &'u str) -> Self {
        Self {
            id: 0,
            shorter_url,
            url,
        }
    }
}
