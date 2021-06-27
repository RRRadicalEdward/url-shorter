use saphir::{
    body::Body,
    controller::{Controller, ControllerEndpoint, EndpointsBuilder},
    http::{Method, StatusCode},
    macros::controller,
    request::Request,
    response::{Builder as HttpResponse, Response},
};
use slog::{debug, info, o, Logger};
use url::Url;

use crate::{
    db::{models, Database},
    utils::{HttpResponseEx, ResultLogger},
    SHORT_URL_COUNT,
};
use std::sync::atomic::Ordering;

pub struct ShorterController {
    database: Database,
    web_logger: Logger,
}

impl ShorterController {
    pub fn new(database: Database, web_logger: Logger) -> Self {
        Self { database, web_logger }
    }

    pub async fn redirect_to_origin_url(&self, req: Request<Body>) -> Result<HttpResponse, HttpResponse> {
        let shorted_url = req.captures().get("shorted_url").ok_or(()).or_bad_request()?;

        let db_logger = self
            .database
            .logger
            .new(o!("check if shorted url exists in DB" => format!("shorter_url: {}", shorted_url)));

        let web_logger = self
            .web_logger
            .new(o!("check if shorted URL exists in DB" => format!("shorter_url: {}", shorted_url)));

        let origin_url = self
            .database
            .get_origin_url(shorted_url.clone())
            .await
            .log_on_err(&db_logger, "Failed to check if shorted URL exists in database")
            .or_bad_request()?;

        debug!(
            web_logger,
            "Found `{}` shorted URL  in the DB and its related to `{}` URL", shorted_url, origin_url
        );

        Ok(Response::<Body>::builder()
            .status(StatusCode::PERMANENT_REDIRECT)
            .header("Location", origin_url))
    }

    pub async fn short_url(&self, mut req: Request) -> Result<HttpResponse, HttpResponse> {
        let url = req.body_mut().take_as::<String>().await.or_bad_request()?;

        let web_logger = self.web_logger.new(o!("Shortening URL: " => url.clone()));

        Url::parse(url.as_str())
            .log_on_err(&web_logger, format!("Got incorrect URL: {}", url).as_str())
            .or_bad_request()?;

        let unique_id = SHORT_URL_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        let shorted_url = radix_fmt::radix_36(unique_id).to_string();

        let url_model = models::Url {
            id: unique_id,
            shorter_url: shorted_url.as_str(),
            url: url.as_str(),
        };

        debug!(self.web_logger, "Got a unique_id: {}", url_model.id);

        let db_logger = self.database.logger.new(o!("Shortening URL: " => url.clone()));
        self.database
            .save_shorter_url(url_model)
            .await
            .log_on_err(&db_logger, "Failed to save shorted URL: {}")
            .or_internal_error()?;

        info!(
            self.web_logger,
            "Shorted `{}` URL to `{}` URL successfully", url, shorted_url
        );

        Ok(Response::<Body>::builder().status(StatusCode::OK).body(shorted_url))
    }
}

impl Controller for ShorterController {
    const BASE_PATH: &'static str = "/";

    fn handlers(&self) -> Vec<ControllerEndpoint<Self>>
    where
        Self: Sized,
    {
        EndpointsBuilder::new()
            .add(Method::GET, "/shorted_url/<shorted_url>", Self::redirect_to_origin_url)
            .add(Method::POST, "/short_url", Self::short_url)
            .build()
    }
}
pub struct HealthController {}

#[controller(name = "health")]
impl HealthController {
    #[get("/")]
    pub async fn health_controller(&self) -> (StatusCode, String) {
        (StatusCode::OK, String::from("It's ALIVE!!!"))
    }
}
