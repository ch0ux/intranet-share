use crate::core::get_sender_running_number;
use crate::core::models::{MessageTransAccept, MessageTransHandle};
use crate::disc::models::CurrentClientInfo;
use crate::models::{CusErr, EmitAsk, FileInfo, IntranetClientInfo, TransInfo, TransType};
use crate::{consts, get_global_ask_map, get_mdns_handle_runtime, get_trans_handle_runtime, utils};
use anyhow::anyhow;
use log::{error, info};
use mdns_sd::ServiceEvent;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

pub struct AppState {
    pub data_current_client: Option<CurrentClientInfo>,
    pub data_intranet_clients: Arc<Mutex<Vec<IntranetClientInfo>>>,
    pub data_files: Vec<FileInfo>,
    pub data_trans_status: Arc<Mutex<HashMap<String, TransInfo>>>,
    pub handle_tauri_app: AppHandle,
}

impl AppState {
    pub fn new(
        handle_tauri_app: AppHandle,
    ) -> anyhow::Result<Self> {
        let app_state = AppState {
            data_current_client: None,
            data_intranet_clients: Arc::new(Mutex::new(vec![])),
            data_files: vec![],
            data_trans_status: Arc::new(Mutex::new(Default::default())),
            handle_tauri_app,
        };

        Ok(app_state)
    }

    pub fn init_data(&mut self) -> anyhow::Result<CurrentClientInfo>{
        let port = utils::get_available_port()?;
        let save_path = dirs::download_dir().unwrap_or_else(Default::default);
        let local_ip = utils::get_local_ip()?.to_string();
        let client_name = format!("{}-{}",sys_info::hostname()?,port);

        if let Ok(mut handle) = get_trans_handle_runtime().lock(){
            // 接收文件 监听处理
            let c_app_handle = self.handle_tauri_app.clone();
            let c_data_trans_status = self.data_trans_status.clone();
            handle.start_receiver(format!("{}:{}",local_ip.clone(), port), move |message|{
                match message {
                    MessageTransHandle::ReceiverAsk { reqs, ask_tx} => {
                        let uid = Uuid::new_v4().to_string();
                        if let Ok(mut d) = get_global_ask_map().lock(){
                            d.insert(uid.clone(), ask_tx);
                        }
                        if let Err(e) = c_app_handle.emit("e_ask", EmitAsk{uid, files:reqs}){
                            error!("app handle emit err {} ", e)
                        }
                    }
                    MessageTransHandle::ReceiverFile { from,file_name,total_len,trans_len, } => {
                        if let Ok(mut dts) = c_data_trans_status.lock(){
                            let key = format!("{}_{}", from, file_name);
                            if let Some(info) = dts.get_mut(&key) {
                                info.total_size = total_len;
                                info.trans_size = trans_len;
                                info.is_done = total_len == trans_len;
                            } else {
                                dts.insert(
                                    key,
                                    TransInfo {
                                        filename: file_name,
                                        from,
                                        to: "".to_string(),
                                        trans_type: TransType::Receive,
                                        total_size: total_len,
                                        trans_size: trans_len,
                                        is_done: false,
                                    },
                                );
                            }
                        }
                    }
                    _ => {}
                }
                if let Ok(dts) = c_data_trans_status.lock(){
                    let trans_status: Vec<TransInfo> = dts.values().cloned().collect();
                    if let Err(e) = c_app_handle.emit("e_trans", trans_status) {
                        error!("app handle emit err {} ", e)
                    }
                }
            })?;
            // 发送文件 监听处理
            let c_app_handle = self.handle_tauri_app.clone();
            let c_data_trans_status = self.data_trans_status.clone();
            handle.start_sender(move|message|{
                if let MessageTransHandle::SendFile {to,file_name,total_len,trans_len, } = message {
                    if let Ok(mut dts) = c_data_trans_status.lock(){
                        let key = format!("{}_{}", to, file_name);
                        if let Some(info)= dts.get_mut(&key) {
                            info.total_size = total_len;
                            info.trans_size = trans_len;
                            info.is_done = total_len == trans_len;
                        } else {
                            dts.insert(key,TransInfo {
                                filename: file_name,
                                from: "".to_string(),
                                to,
                                trans_type: TransType::Send,
                                total_size: total_len,
                                trans_size: trans_len,
                                is_done: false}
                            );
                        }
                    }
                }
                if let Ok(dts) = c_data_trans_status.lock(){
                    let trans_status: Vec<TransInfo> = dts.values().cloned().collect();
                    if let Err(e) = c_app_handle.emit("e_trans", trans_status) {
                        error!("app handle emit err {} ", e)
                    }
                }
            })?;
        }

        if let Ok(mut handle) = get_mdns_handle_runtime().lock(){
            let c_app_handle = self.handle_tauri_app.clone();
            let c_data_intranet_clients = self.data_intranet_clients.clone();
            handle.set_call_fn(Box::new(
                move |resolved_service|{

                    match resolved_service {
                        ServiceEvent::ServiceResolved(rs) => {
                            info!("mdns resolved service {:?}", rs);
                            if let Ok(mut clients) = c_data_intranet_clients.lock() &&
                                clients.iter().filter(|c|c.name.eq(rs.get_fullname())).count() == 0
                                && let Some(ip) = rs.get_addresses_v4().iter().next().map(|ip|ip.to_string()){
                                clients.push(IntranetClientInfo{
                                    name: rs.fullname,
                                    ip,
                                    port: rs.port,
                                    selected: false,
                                    auto_find: true,
                                })
                            }

                        }
                        ServiceEvent::ServiceRemoved(service_type, full_name) => {
                            if let Ok(mut clients) = c_data_intranet_clients.lock() &&
                                service_type.eq(consts::SERVICE_TYPE){
                                clients.retain(|c|!c.name.eq(&full_name));
                            }
                        }
                        ServiceEvent::ServiceFound(service_type, full_name) => {
                            println!("service found{} -> {}", service_type, full_name)
                        }
                        _ => {}
                    }

                    if let Ok(clients) = c_data_intranet_clients.lock(){

                        let c_s = clients.iter().map(|c|c.to_owned()).collect::<Vec<IntranetClientInfo>>();

                        if let Err(e) = c_app_handle.emit("e_intranet_clients",c_s) {
                            error!("app handle emit err {} ", e)
                        }
                    }
                }
            ));

            if let Err(e)= handle.register(client_name.clone(), local_ip.clone(), port){
                error!("register mdns err {}", e)
            }
        }

        let current_client = CurrentClientInfo {
            local_ip,
            port,
            name: client_name ,
            save_path,
        };

        self.data_current_client = Some(current_client.clone());

        self.emit_app_info()?;

        Ok(current_client)
    }

    pub fn emit_app_info(&mut self) -> anyhow::Result<()> {
        self.handle_tauri_app.emit("e_current_client", self.data_current_client.clone())?;
        self.handle_tauri_app.emit(
            "e_intranet_clients",
            self.data_intranet_clients.clone(),
        )?;
        self.handle_tauri_app.emit("e_files", self.data_files.clone())?;

        Ok(())
    }
}

#[tauri::command]
pub fn init(state: State<'_, Arc<Mutex<AppState>>>) -> Result<CurrentClientInfo, CusErr> {
    let mut app_state = state.lock()?;
    match app_state.data_current_client {
        None => {
            let c = app_state.init_data()?;
            Ok(c)
        }
        Some(ref c) => Ok(c.clone())
    }
}

#[tauri::command]
pub fn pick_selected_path(
    state: State<'_, Arc<Mutex<AppState>>>,
    save_dir: PathBuf,
) -> Result<(), CusErr> {

    if let Ok(mut handle) = get_trans_handle_runtime().lock(){
        handle.change_save_dir(save_dir.clone());
    }

    if let Ok(mut app_state) = state.lock(){
        if let Some(ref mut data_current_client) = app_state.data_current_client{
            data_current_client.save_path = save_dir;
        }
        app_state.emit_app_info()?;
    }

    Ok(())
}

#[tauri::command]
pub fn add_client(
    state: State<'_, Arc<Mutex<AppState>>>,
    mut client: IntranetClientInfo,
) -> Result<(), CusErr> {
    let mut app_state = state.lock()?;

    if app_state.data_intranet_clients.lock()?.iter().filter(|i| i.name.eq(&client.name))
        .count() > 0
    {
        return Err(CusErr::from(anyhow!("已存在同名客户端")));
    }

    client.auto_find = false;

    app_state.data_intranet_clients.lock()?.push(client);

    app_state.emit_app_info()?;

    Ok(())
}

#[tauri::command]
pub fn selected_client(
    state: State<'_, Arc<Mutex<AppState>>>,
    name: String,
    selected: bool,
) -> Result<(), CusErr> {
    let mut app_state = state.lock()?;

    if let Some(c) = app_state
        .data_intranet_clients
        .lock()?
        .iter_mut()
        .find(|i| i.name.eq(&name))
    {
        c.selected = selected
    }

    app_state.emit_app_info()?;

    Ok(())
}

#[tauri::command]
pub fn remove_client(
    state: State<'_, Arc<Mutex<AppState>>>,
    name: String,
) -> Result<(), CusErr> {
    if let Ok(mut app_state) = state.lock(){
        // 保留非name的客户端
        app_state.data_intranet_clients.lock()?.retain(|c|{
            !c.name.eq(&name)
        });

        app_state.emit_app_info()?;
    }
    Ok(())
}

#[tauri::command]
pub fn add_file(
    state: State<'_, Arc<Mutex<AppState>>>,
    files: Vec<String>,
) -> Result<(), CusErr> {
    let mut app_state = state.lock()?;

    // 校验重复 不添加
    let o_files = app_state
        .data_files
        .iter()
        .map(|f| f.file.clone())
        .collect::<Vec<PathBuf>>();

    let n_files: Vec<PathBuf> = files
        .iter()
        .map(PathBuf::from)
        .filter(|f| !o_files.contains(f))
        .collect();

    if !n_files.is_empty() {
        for f in n_files {
            let filename = f
                .file_name()
                .ok_or(CusErr::from(anyhow!("文件名获取失败")))?
                .to_string_lossy()
                .to_string();
            let f_size = utils::get_file_size(&f)?;
            app_state.data_files.push(FileInfo {
                filename,
                file: f.to_owned(),
                size: f_size,
            })
        }
    }

    app_state.emit_app_info()?;
    Ok(())
}

#[tauri::command]
pub fn remove_file(
    state: State<'_, Arc<Mutex<AppState>>>,
    name: String,
) -> Result<(), CusErr> {
    let mut app_state = state.lock()?;

    if let Some(idx) = app_state
        .data_files
        .iter()
        .position(|f| f.filename.eq(&name))
    {
        app_state.data_files.remove(idx);
    }

    app_state.emit_app_info()?;
    Ok(())
}

/*
项目已启动 就会注册mdns，但是是否搜索局域网数据 需要按钮出发
*/
#[tauri::command]
pub fn search_client() -> Result<(), CusErr>{
    if let Ok(mut handle) = get_mdns_handle_runtime().lock(){
        if let Err(e) = handle.start_listen(){
            error!("start mdns err {}", e)
        }
    }
    Ok(())
}

#[tauri::command]
pub fn close_search_client() -> Result<(), CusErr> {
    if let Ok(mut handle) = get_mdns_handle_runtime().lock(){
        if let Err(e) = handle.close_listen() {
            error!("start mdns err {}", e)
        }
    }
    Ok(())
}

#[tauri::command]
pub fn start_trans(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), CusErr> {
    // 根据文件和选中的客户端生成传输列表
    // 回调更新对应的文件状态

    //任务发起改为 channel

    //校验 ,如果正在执行任务 则不执行
    if let Ok(srn) = get_sender_running_number().lock() &&
        !srn.is_empty(){
            return Err(CusErr::from(anyhow!("任务正在执行中")));
        }


    let mut app_state = state.lock()?;

    if let Ok(mut trans_d) = app_state.data_trans_status.lock() {
        // 使用 retain 一次性过滤掉已完成的任务
        trans_d.retain(|_,v|!v.is_done);
    }

    let clients: Vec<IntranetClientInfo> = app_state
        .data_intranet_clients
        .lock()?
        .iter()
        .filter(|c| c.selected)
        .cloned()
        .collect();
    let files: Vec<PathBuf> = app_state
        .data_files
        .iter()
        .map(|f| f.file.clone())
        .collect();

    if !clients.is_empty() && !files.is_empty() {

        for c in clients {
            let target = format!("{}:{}", c.ip, c.port);
            if let Ok(mut handle) = get_trans_handle_runtime().lock(){
                if let Err(e) = handle.send_files(MessageTransAccept::CreateJob {
                        to: target,
                        multi_paths: files.clone(),})
                {
                    error!("发送任务失败:{}", e);
                    return Err(CusErr::from(anyhow!("发送任务失败:{}", e)));
                }
            }

        }

        // 清空文件列表
        app_state.data_files.clear();
        // 重置已选择的客户端
        app_state.data_intranet_clients.lock()?.iter_mut().for_each(|c|c.selected = false);

    }

    app_state.emit_app_info()?;

    Ok(())
}

#[tauri::command]
pub async fn answer_accept(uid: String, answer: bool)-> Result<(), CusErr>{

    let sender = {
        get_global_ask_map().lock()?.remove(&uid)
    };
    if let Some(s) = sender{
        if let Err(e) = s.send(answer).await{
            error!("send answer error:{}", e);
        }
    }
    Ok(())
}
