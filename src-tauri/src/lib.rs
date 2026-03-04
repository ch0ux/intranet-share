use crate::commands::AppState;
use crate::core::handle::TransHandle;
use crate::disc::discover::MdnsActorState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{App, Manager};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

mod commands;
mod core;
mod disc;
mod models;
mod utils;
mod consts;

// 发送接收的处理运行时逻辑
static TRANS_HANDLE_RUNTIME: OnceLock<Mutex<TransHandle>> = OnceLock::new();

fn get_trans_handle_runtime() -> &'static Mutex<TransHandle> {
    TRANS_HANDLE_RUNTIME.get_or_init(||{
        Mutex::new(TransHandle::init())
    })
}

static MDNS_HANDLE_RUNTIME: OnceLock<Mutex<MdnsActorState>> = OnceLock::new();

fn get_mdns_handle_runtime() -> &'static Mutex<MdnsActorState> {
    MDNS_HANDLE_RUNTIME.get_or_init(||{
        let mdns = MdnsActorState::new().expect("Failed to create mdns handle");
        Mutex::new(mdns)
    })
}

// static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

// fn tokio_runtime_init() {
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

static GLOBAL_ASK_MAP: OnceLock<Mutex<HashMap<String, mpsc::Sender<bool>>>> = OnceLock::new();

pub fn get_global_ask_map() -> &'static Mutex<HashMap<String, Sender<bool>>> {
    GLOBAL_ASK_MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn init(app:&mut App) -> anyhow::Result<()>{
    let handle_tauri_app = app.handle().clone();

    let app_state = AppState::new(handle_tauri_app, )?;

    app.manage(Arc::new(Mutex::new(app_state)));
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            init(app)?;
            // info!("init time: {}ms",SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() - now);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::init,
            commands::pick_selected_path,
            commands::add_client,
            commands::selected_client,
            commands::remove_client,
            commands::add_file,
            commands::remove_file,
            commands::start_trans,
            commands::answer_accept,
            commands::search_client,
            commands::close_search_client,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
