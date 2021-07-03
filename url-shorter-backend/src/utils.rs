use saphir::{
    body::Body,
    http::StatusCode,
    response::{Builder as HttpResponse, Response},
};
use slog::{error, o, Drain, Logger};
use slog_async::{Async, OverflowStrategy};
use slog_term::{FullFormat, TermDecorator};
use std::fmt;

const DEFAULT_LOG_CHAN_SIZE: usize = 256;

pub fn build_logger() -> Logger {
    let term_decorator = TermDecorator::new().build();
    let term_fmt = FullFormat::new(term_decorator).use_local_timestamp().build().fuse();

    let drain = Async::new(slog_envlogger::new(term_fmt).fuse())
        .chan_size(DEFAULT_LOG_CHAN_SIZE)
        .overflow_strategy(OverflowStrategy::DropAndReport)
        .build()
        .fuse();

    Logger::root(
        drain,
        o!("backend" => slog::FnValue(
            move |info| {
                format!("[{}]", info.module())
            }
        )),
    )
}

pub trait ResultLogger {
    fn log_on_err(self, logger: &Logger, message: &str) -> Self;
}

impl<T, E> ResultLogger for Result<T, E>
where
    E: fmt::Display,
{
    fn log_on_err(self, logger: &Logger, message: &str) -> Self {
        if let Err(err) = self.as_ref() {
            error!(logger, "{} -> {}", message, err);
        }

        self
    }
}

pub trait HttpResponseEx<T> {
    fn or_bad_request(self) -> Result<T, HttpResponse>;
    fn or_internal_error(self) -> Result<T, HttpResponse>;
}

impl<T, E> HttpResponseEx<T> for Result<T, E> {
    fn or_bad_request(self) -> Result<T, HttpResponse> {
        self.map_err(|_| Response::<Body>::builder().status(StatusCode::BAD_REQUEST))
    }

    fn or_internal_error(self) -> Result<T, HttpResponse> {
        self.map_err(|_| Response::<Body>::builder().status(StatusCode::INTERNAL_SERVER_ERROR))
    }
}
