use anyhow::Context;
use mongodb::{
    bson::{doc, Document},
    options::ClientOptions,
    Client, Database as MongoDatabase,
};
use slog::{info, Logger};
use std::{
    env,
    error::Error,
    fmt::{self, Formatter},
};

pub mod models;
use models::Url;

const ARG_DATABASE_URL: &str = "DATABASE_URL";

#[derive(Clone)]
pub struct Database {
    pub inner: MongoDatabase,
    pub logger: Logger,
}

impl Database {
    pub async fn establish_connection(logger: Logger) -> anyhow::Result<Database> {
        let database_url = match (dotenv::var(ARG_DATABASE_URL), env::var(ARG_DATABASE_URL)) {
            (Ok(database_url), _) => {
                info!(logger, "Using DATABASE_URL from .env: {}", database_url);
                database_url
            }
            (_, Ok(database_url)) => {
                info!(
                    logger,
                    "Using DATABASE_URL from the environment variable: {}", database_url
                );
                database_url
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "No DATABASE_URL specified in .env neither in the environment variable"
                ))
            }
        };

        let options = ClientOptions::parse(&database_url)
            .await
            .with_context(|| "Failed to parse DATABASE_URL")?;
        let options = ClientOptions::builder()
            .credential(options.credential)
            .max_pool_size(Some(8))
            .min_pool_size(Some(0))
            .build();

        let client = Client::with_options(options).with_context(|| "Failed to create a client connected to MongoDB")?;
        let database = client.database("url-shorter");

        info!(logger, "Connected to Database");

        Ok(Self {
            inner: database,
            logger,
        })
    }

    pub async fn save_shorter_url(&self, url_model: Url<'_>) -> anyhow::Result<()> {
        let urls = self.inner.collection("urls");
        let id = url_model.id;
        urls.insert_one(Document::from(url_model), None).await?;

        urls.find_one(
            doc! {
                "id": id
            },
            None,
        )
        .await?
        .ok_or_else(|| {
            NotFound {
                message: "Failed to find saved shorter URL".to_string(),
            }
            .into()
        })
        .map(|_| ())
    }

    pub async fn get_origin_url(&self, short_url: String) -> anyhow::Result<String> {
        let urls = self.inner.collection("urls");
        urls.find_one(
            doc! {
                "shorter_url": short_url
            },
            None,
        )
        .await?
        .map(|mut doc| doc.entry(String::from("url")).key().to_string())
        .ok_or_else(|| {
            NotFound {
                message: "Failed to find origin URL by its shorted variant".to_string(),
            }
            .into()
        })
    }
}

#[derive(Debug)]
struct NotFound {
    message: String,
}

impl Error for NotFound {}

impl fmt::Display for NotFound {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use std::{process::Command, sync::Arc};
    use tokio::{runtime::Runtime, sync::Mutex};

    lazy_static! {
        pub static ref DATABASE: Arc<Mutex<Database>> = {
            let mut runtime = Runtime::new().unwrap();

            let exit_status = Command::new("sh")
                .arg("tests/startup-test-mongodb.sh")
                .spawn()
                .unwrap()
                .wait()
                .unwrap();

            if !exit_status.success() {
                panic!("Failed to execute the command");
            }

            dotenv::from_filename("tests/test.env").ok();

            let logger = crate::utils::build_logger();

            let db = runtime.block_on(Database::establish_connection(logger)).unwrap();

            Arc::new(Mutex::new(db))
        };
    }

    #[test]
    fn startup_database() {
        let _ = DATABASE.clone();
    }

    #[tokio::test]
    async fn save_url_into_database() {
        let database = DATABASE.clone();

        let database = &mut *database.lock().await;
        database
            .save_shorter_url(Url::new("http://hello-world").unwrap())
            .await
            .unwrap();
    }
}
