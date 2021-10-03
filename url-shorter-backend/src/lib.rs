pub mod db;
pub mod utils;
pub mod web;

use lazy_static::lazy_static;
use std::sync::atomic::AtomicU32;

lazy_static! {
    pub static ref SHORT_URL_COUNT: AtomicU32 = AtomicU32::new(0);
}
