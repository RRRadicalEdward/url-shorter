pub mod db;
pub mod utils;
pub mod web;

use lazy_static::lazy_static;
use std::sync::atomic::AtomicU64;

lazy_static! {
    pub static ref SHORT_URL_COUNT: AtomicU64 = AtomicU64::new(0);
}
