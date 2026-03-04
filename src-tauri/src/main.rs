// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() {
    // 首先尝试 tauri自行处理运行时
    tauri::async_runtime::set(tokio::runtime::Handle::current());
    intranet_share_lib::run()
}
