use std::fs;
use std::net::{IpAddr, TcpListener, UdpSocket};
use std::path::PathBuf;

pub fn get_file_size(p: &PathBuf) -> anyhow::Result<u64> {
    let metadata = fs::metadata(p)?;
    anyhow::Ok(metadata.len())
}

pub fn get_available_port() -> anyhow::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    anyhow::Ok(listener.local_addr()?.port())
}

pub fn get_local_ip() -> anyhow::Result<IpAddr>{
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    Ok(socket.local_addr()?.ip())
}