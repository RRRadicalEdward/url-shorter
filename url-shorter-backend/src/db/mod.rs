use std::convert::{Into, TryInto};

use diesel::{
    dsl::max,
    insert_into,
    query_dsl::{
        filter_dsl::FilterDsl,
        methods::{OrderDsl, SelectDsl},
    },
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
    ExpressionMethods, OptionalExtension, RunQueryDsl,
};
use slog::{debug, error, info, trace, Logger};

pub mod models;
pub mod schema;

use models::Url;
use schema::urls;

#[derive(Clone)]
pub struct Database {
    pub pool: Pool<ConnectionManager<SqliteConnection>>,
    pub logger: Logger,
}

impl Database {
    pub fn establish_connection(logger: Logger) -> anyhow::Result<Database> {
        let database_url = dotenv::var("DATABASE_URL").map_err(|err| {
            error!(logger, "Failed to fetch database url: {}", err);
            err
        })?;

        let pool = Pool::builder()
            .max_size(16)
            .build(ConnectionManager::<SqliteConnection>::new(database_url))
            .map_err(|err| {
                error!(logger, "Failed to create DB pool:{}", err);
                err
            })?;

        info!(logger, "Connected to Database");
        Ok(Self { pool, logger })
    }

    pub fn get_new_unique_id(&self) -> anyhow::Result<u32> {
        use self::urls::dsl::*;

        debug!(self.logger, "Getting a new unique id...");

        let connection = self.pool.get().expect("Pool should not panic");

        let query_result: Option<i64> = urls
            .order(id)
            .select(max(id))
            .first::<Option<_>>(&connection)
            .optional()
            .map_err(Into::<anyhow::Error>::into)?
            .flatten();

        let unique_id = match query_result {
            Some(max_id) => TryInto::<u32>::try_into(max_id).expect("Should not panic") + 1,
            None => 0,
        };

        debug!(self.logger, "Successfully get a unique id");
        trace!(self.logger, "A new unique id:{}", unique_id);

        Ok(unique_id)
    }

    pub fn save_shorter_url(&self, url_model: Url) -> anyhow::Result<()> {
        use self::urls::dsl::*;

        let connection = self.pool.get().expect("Pool should not panic");

        insert_into(urls)
            .values(&(shorter_url.eq(url_model.shorter_url), url.eq(url_model.url)))
            .execute(&connection)
            .map_err(Into::<anyhow::Error>::into)?;

        Ok(())
    }

    pub fn get_origin_url(&self, short_url: String) -> anyhow::Result<Option<String>> {
        use self::urls::dsl::*;

        let connection = self.pool.get().expect("Pool should not panic");

        Ok(urls
            .order(id)
            .filter(shorter_url.eq(short_url))
            .first::<(i64, String, String)>(&connection)
            .optional()
            .map_err(Into::<anyhow::Error>::into)?
            .map(|(_, _, origin_url)| origin_url))
    }
}
