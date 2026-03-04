use crate::core::models::ReqTransferData;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::PoisonError;

#[derive(Debug, thiserror::Error)]
pub enum CusErr {
    #[error(transparent)]
    AnyHowError(#[from] anyhow::Error),
    #[error("Mutex poison error")]
    PoisonError,
}

// 实现从 PoisonError<Mutex<T>> 到 CusError 的转换
impl<T> From<PoisonError<T>> for CusErr {
    fn from(_error: PoisonError<T>) -> Self {
        CusErr::PoisonError
    }
}

impl serde::Serialize for CusErr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntranetClientInfo {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub selected: bool,
    pub auto_find: bool, // 是否自动查找
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub filename: String,
    pub file: PathBuf,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransType {
    Send,
    Receive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransInfo {
    pub filename: String,
    pub from: String,
    pub to: String,
    pub trans_type: TransType, // send|receive
    pub total_size: u64,
    pub trans_size: u64,
    pub is_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitAsk {
    pub uid: String,
    pub files: Vec<ReqTransferData>,
}