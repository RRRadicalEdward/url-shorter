use anyhow::Context;
use dotenv::dotenv;
use lib::{
    db::Database,
    utils,
    web::{HealthController, ShorterController},
};
use saphir::prelude::*;
use slog::{info, o};
use std::convert::Into;

const DEFAULT_IP_ADDR: &str = "127.0.0.1:5050";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let logger = utils::build_logger();
    let web_logger = logger.new(o!("web" => "request"));
    let db_logger = logger.new(o!("Database" => "connection"));

    dotenv().ok();
    let database = Database::establish_connection(db_logger)
        .await
        .with_context(|| "Failed to connect to MongoDB")?;

    let server = Server::builder()
        .configure_listener(|l| l.interface(DEFAULT_IP_ADDR).server_name("url-shorter"))
        .configure_router(|router| {
            info!(web_logger, "Running web service on {}", DEFAULT_IP_ADDR);

            router
                .controller(HealthController {})
                .controller(ShorterController::new(database, web_logger))
        })
        .build();

    server.run().await.map_err(Into::<anyhow::Error>::into)
}
