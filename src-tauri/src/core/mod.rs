use std::sync::{Mutex, OnceLock};

mod receiver;
mod sender;
pub mod handle;
mod utils;
mod consts;
pub mod models;

type CusResult<T> = anyhow::Result<T>;

static SENDER_RUNNING_NUMBER: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

pub fn get_sender_running_number() -> &'static Mutex<Vec<String>>{
    SENDER_RUNNING_NUMBER.get_or_init(||{
       Mutex::new(Vec::new())
    })
}

// static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
//
// pub fn tokio_runtime_init() {
//     TOKIO_RUNTIME
//         .get_or_init(|| tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"));
// }
// pub fn tokio_handle() -> Handle {
//     TOKIO_RUNTIME
//         .get()
//         .expect("tokio runtime not initialized")
//         .handle()
//         .clone()
// }