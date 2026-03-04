use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Serialize, Deserialize, Debug,Clone,PartialEq)]
pub enum TransferDecision {
    Accept,
    Reject,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct FileMetadata{
    pub filename: String, // 文件名
    pub buf: Vec<u8>, // 文件内容
    pub hash:String, // 文件的哈希值
    pub total_len: u64, // 文件总长度
    pub trans_len: u64, // 已传输的长度
    pub current_len: u64 // 当前已传输的长度
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ReqTransferData{
    pub filename: String,
}

// 👇 新增：控制会话流程
#[derive(Serialize, Deserialize, Debug)]
pub enum SessionMessage {
    // 发送方：请求一个文件
    ReqTransfer(Vec<ReqTransferData>),
    // 接收方：对当前文件的决策
    FileDecision{
        decision:bool
    },
    // 发送方：开始一个新文件
    BeginFile(FileMetadata),
    // 发送方：所有文件已发送完毕
    EndSession,
}

#[derive(Debug,Clone)]
pub enum MessageTransHandle{
    ReceiverAsk{reqs:Vec<ReqTransferData>, ask_tx: mpsc::Sender<bool>},
    ReceiverFile{from: String,file_name:String,total_len: u64,trans_len: u64,},
    SendFile{to: String,file_name:String,total_len: u64,trans_len: u64,},
}

#[derive(Debug, Clone)]
pub enum MessageTransAccept{
    CreateJob{to: String, multi_paths: Vec<PathBuf>},
    Shutdown
}