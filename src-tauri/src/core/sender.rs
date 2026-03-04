use crate::core::models::{FileMetadata, MessageTransAccept, MessageTransHandle, ReqTransferData, SessionMessage, TransferDecision};
use crate::core::utils;
use crate::core::{get_sender_running_number, CusResult};
use anyhow::anyhow;
use log::{error, info};
use serde::Serialize;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub fn spawn_sender_actor(buffer_size:usize) -> (
    broadcast::Sender<MessageTransAccept>,
    broadcast::Receiver<MessageTransHandle>,
    CancellationToken) {
    let (accept_sender, mut accept_receiver) = broadcast::channel::<MessageTransAccept>(buffer_size);
    let (handle_sender, handle_receiver) = broadcast::channel::<MessageTransHandle>(buffer_size);
    let cancellation_token = CancellationToken::new();

    // 克隆 cancellation_token 用于子任务
    let c_cancellation_token = cancellation_token.clone();
    tokio::spawn(async move {
        loop {
            match accept_receiver.recv().await {
                Ok(message) => {
                    let handle_sender = handle_sender.clone();
                    let c_token = c_cancellation_token.clone(); // 使用克隆的 token
                    match message {
                        MessageTransAccept::CreateJob { to, multi_paths } => {
                            info!("sender actor create job to {}", to);
                            tokio::spawn(async move {
                                // 创建随机 id
                                let rand_id = Uuid::new_v4().to_string();
                                // 添加运行中的任务
                                if let Ok(mut srn) = get_sender_running_number().lock(){
                                    srn.push(rand_id.clone())
                                }
                                if let Err(e) = send_files(&to, &multi_paths, handle_sender, c_token).await {
                                    error!("sender actor error {}", e)
                                }
                                // 移除运行中的任务
                                if let Ok(mut srn) = get_sender_running_number().lock(){
                                    srn.retain(|s|!s.eq(&rand_id))
                                }
                            });
                        }
                        MessageTransAccept::Shutdown => {
                            info!("sender actor shutdown");
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("sender actor error {}", e)
                }
            }
        }
        info!("sender actor stopped");
    });

    (accept_sender, handle_receiver, cancellation_token) // 返回原始 token
}

pub async  fn send_files(
    addr:&str,
    files:&[PathBuf],
    handle_sender: broadcast::Sender<MessageTransHandle>,
    cancellation_token: CancellationToken
) -> CusResult<()> {

    let filenames = files.iter()
        .filter_map(|f|{
            // 安全地提取文件名，并处理可能的 None 情况
            f.file_name()
                .and_then(|name| name.to_str()) // 合并字符串转换
                .map(|f| ReqTransferData { filename:f.to_owned() })     // 构造结构体
        })
        .collect::<Vec<ReqTransferData>>();

    info!("sending files to {:?}", filenames);

    let trans_msg = SessionMessage::ReqTransfer(filenames);
    let mut stream = TcpStream::connect(addr).await?;
    send_message(&mut stream, &trans_msg).await?;

    let decision = recv_decision(&mut stream).await?;
    match decision {
        TransferDecision::Accept => {
            for path in files.iter(){
                trans_file(&mut stream, addr, path, handle_sender.clone(),cancellation_token.clone()).await?;
            }
        }
        TransferDecision::Reject => {
            info!("transfer rejected");
            return Ok(())
        }
    }

    Ok(())
}


async fn trans_file(
    stream: &mut TcpStream,
    addr:&str,
    file: &PathBuf,
    handle_sender :broadcast::Sender<MessageTransHandle>,
    cancellation_token: CancellationToken
) -> CusResult<()> {

    let filename = file.file_name()
        .and_then(|name| name.to_str())
        .ok_or(anyhow!("Invalid filename"))?
        .to_string();
    let total_len = utils::get_file_size(file)?;

    info!("transferring get file size {}", total_len);

    // 创建文件读取流
    let f = tokio::fs::File::open(file).await?;
    let mut reader = BufReader::new(f);

    let mut buf = vec![0u8; 256 * 1024];// 256K
    let mut trans_len = 0u64;

    //此处添加 停止的标记
    loop{
        tokio::select! {
            _ =cancellation_token.cancelled() => {
                info!("sender actor cancelled");
                break;
            },
            read_res = reader.read(&mut buf) => {
                let n = read_res?;
                if n ==0 {
                    break
                }
                trans_len += n as u64;

                let hash = utils::calc_hex(&buf[..n]).await;

                // 发送文件元数据
                let trans_msg = SessionMessage::BeginFile(FileMetadata{
                    filename:filename.clone(),
                    buf: buf[..n].to_vec(),// 只克隆有效数据部分
                    hash,
                    total_len,
                    trans_len,
                    current_len: n as u64,
                });
                send_message(stream, &trans_msg).await?;

                // 上报文件传输消息
                handle_sender.send(MessageTransHandle::SendFile {
                    to: addr.to_string(),
                    file_name:filename.clone(),
                    total_len,
                    trans_len,})?;
                    }
        }
    }

    // 结束会话
    send_message(stream, &SessionMessage::EndSession).await?;

    reader.shutdown().await?;
    Ok(())
}

async fn recv_decision(stream: &mut TcpStream) -> CusResult<TransferDecision>{
    let mut buf_reader = BufReader::new(stream);
    let mut line = String::new();
    buf_reader.read_line(&mut line).await?;

    let msg: TransferDecision = serde_json::from_str(line.trim())?;

    Ok(msg)
}

async fn send_message<T>(stream: &mut TcpStream, message: &T) -> CusResult<()>
where
    T: Serialize
{

    let json = serde_json::to_string(message)?;
    stream.write_all(json.as_bytes()).await?;
    stream.write_u8(b'\n').await?;

    Ok(())
}