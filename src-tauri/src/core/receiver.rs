use std::net::SocketAddr;
use crate::core::{utils, CusResult};
use crate::core::models::{FileMetadata, MessageTransHandle, ReqTransferData, SessionMessage, TransferDecision};
use crate::core::consts;
use serde::Serialize;
use std::option::Option;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::Sender;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio_util::sync::CancellationToken;
use log::{error, info};

pub fn spawn_receiver_actor(addr: String, buffer_size:usize,save_path: Arc<Mutex<PathBuf>>) -> (broadcast::Receiver<MessageTransHandle>, CancellationToken) {
    
    let (sender,receiver) = broadcast::channel::<MessageTransHandle>(buffer_size);
    let cancellation_token = CancellationToken::new();

    // 在最外层使用 tokio_handle() 确定上下文的 tokio 运行时即可
    tokio::spawn(start_listener(addr,sender,cancellation_token.clone(),save_path));

    (receiver, cancellation_token)
}


pub async fn start_listener(
    addr:String,
    sender:Sender<MessageTransHandle>,
    cancellation_token: CancellationToken,
    save_path: Arc<Mutex<PathBuf>>
) -> CusResult<()> {

    let tcp_listener = TcpListener::bind(&addr).await?;
    info!("Listening on {}", addr);

    let mut child_task = tokio::task::JoinSet::new();

    loop{
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                info!("Cancellation signal received, shutting down listener.");
                break;
            }
            accept_result = tcp_listener.accept() => {
                match accept_result {
                    Ok((socket, s_addr))  => {
                        let c_sender = sender.clone();
                        let c_save_path = save_path.clone();
                        child_task.spawn(async move {
                            if let Err(e) = handle_socket(socket, c_save_path,s_addr, c_sender).await{
                                error!("Error: {:?}", e)
                            }
                        });
                    }
                    Err(e) => {
                        error!("Error: {:?}", e);
                    },
                }
            }
        }
    }

    while let Some(res) = child_task.join_next().await{
        if let Err(e) = res{
            error!("Task failed: {:?}", e);
        }
    }

    anyhow::Ok(())
}

pub async fn handle_socket(
    tcp_stream: TcpStream,
    m_save_dir: Arc<Mutex<PathBuf>>,
    s_addr: SocketAddr,
    sender:Sender<MessageTransHandle>
) -> CusResult<()>{

    let mut buffer_reader = BufReader::new(tcp_stream);
    let mut read_line = String::new();

    let mut file_opt:Option<File> = None;

    loop {
        read_line.clear();
        if buffer_reader.read_line(&mut read_line).await? == 0{
            break
        }

        let message = serde_json::from_str::<SessionMessage>(read_line.as_str())?;

        match message {
            SessionMessage::ReqTransfer(reqs) => {
                let reply = decide_to_accept(reqs, sender.clone()).await?;
                info!("Reply: {:?}", reply);
                send_message(&mut buffer_reader, &reply).await?;
            }
            SessionMessage::BeginFile(file_metadata) => {
                if file_opt.is_none() {
                    let save_dir  = m_save_dir.lock().await;
                    let file_path = save_dir.join(&file_metadata.filename);
                    file_opt = Some(fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(file_path)
                        .await?)
                }
                if let Some(ref mut f) = file_opt {
                    receive_file(f, &file_metadata).await?;
                    sender.send(MessageTransHandle::ReceiverFile{
                        from: s_addr.to_string(),
                        file_name: file_metadata.filename,
                        total_len: file_metadata.total_len,
                        trans_len: file_metadata.trans_len,
                    })?;
                }
            }
            SessionMessage::EndSession => {
                if let Some(ref mut f) = file_opt.take(){
                    f.shutdown().await?;
                    info!("EndSession File saved: {}", f.metadata().await?.len());
                }
                break
            }
            _ => {
                break
            }
        }
    }

    buffer_reader.get_mut().shutdown().await?;
    anyhow::Ok(())
}


// 询问是否接受
async fn decide_to_accept(
    reqs:Vec<ReqTransferData>,
    trans_msg_sen :broadcast::Sender<MessageTransHandle>
) -> anyhow::Result<TransferDecision> {
    let (ask_tx, mut ask_rx) = mpsc::channel::<bool>(2);

    // 上报消息确认
    trans_msg_sen.send(MessageTransHandle::ReceiverAsk { reqs, ask_tx: ask_tx.clone() })?;

    match tokio::time::timeout(Duration::from_secs(consts::MAX_TIME_OUT_SEC), ask_rx.recv()).await {
        Ok(Some(true)) => Ok(TransferDecision::Accept),
        Ok(Some(false)) | Ok(None) => {
            Ok(TransferDecision::Reject)
        }
        Err(e) => {
            error!("Error: {:?}", e);
            Ok(TransferDecision::Reject)
        }
    }
}

pub async fn send_message<T>(
    buffer_reader:&mut BufReader<TcpStream>,
    message :&T) -> anyhow::Result<()>
where
    T: Serialize,
{
    let json = serde_json::to_string(message)?;
    buffer_reader.get_mut().write_all(json.as_bytes()).await?;
    buffer_reader.get_mut().write_u8(b'\n').await?;
    Ok(())
}

async fn receive_file(f: &mut File, fl:&FileMetadata) -> CusResult<()> {
    let actual = utils::calc_hex(&fl.buf[..fl.current_len as usize]).await;
    if actual != fl.hash{
        return Err(anyhow::anyhow!("hash not match"))
    }

    // 写入文件
    f.write_all(&fl.buf[..fl.current_len as usize]).await?;
    Ok(())
}