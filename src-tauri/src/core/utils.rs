use blake3::Hasher;
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;

pub fn get_file_size(p:&PathBuf) -> anyhow::Result<u64>{
    let metadata = fs::metadata(p)?;
    anyhow::Ok(metadata.len())
}

#[allow(dead_code)]
pub fn get_available_port() -> anyhow::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    anyhow::Ok(listener.local_addr()?.port())
}

pub async fn calc_hex(buf: &[u8]) -> String{
    let mut hasher = Hasher::new();
    hasher.update(buf);
    hasher.finalize().to_hex().to_string()
}
