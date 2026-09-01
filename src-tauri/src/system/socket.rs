//! 文件名称：socket.rs
//!
//! 文件功能：
//! 通过系统网络表枚举本机监听套接字。
//!
//! 主要职责：
//! - 读取 TCP 监听端口与 UDP 绑定端口
//! - 附带占用该套接字的 PID
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState,
};

use crate::errors::AppError;

/// 一条系统套接字记录，尚未补充进程名称。
#[derive(Debug, Clone)]
pub struct SocketRecord {
    pub port: u16,
    pub protocol: String,
    pub status: String,
    pub address: String,
    pub pid: u32,
}

/// 扫描当前系统 TCP 监听端口和 UDP 绑定端口。
///
/// 返回：端口、协议、状态、绑定地址、PID。
pub fn list_sockets() -> Result<Vec<SocketRecord>, AppError> {
    let address_family = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let protocol = ProtocolFlags::TCP | ProtocolFlags::UDP;
    let sockets = get_sockets_info(address_family, protocol)
        .map_err(|error| AppError::SystemError(error.to_string()))?;

    let mut records = Vec::new();
    for socket in sockets {
        let pid = socket.associated_pids.first().copied().unwrap_or(0);
        match socket.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) => {
                if tcp.state != TcpState::Listen {
                    continue;
                }
                records.push(SocketRecord {
                    port: tcp.local_port,
                    protocol: "TCP".to_string(),
                    status: "LISTEN".to_string(),
                    address: format_listen_address(&tcp.local_addr.to_string(), tcp.local_port),
                    pid,
                });
            }
            ProtocolSocketInfo::Udp(udp) => {
                if udp.local_port == 0 {
                    continue;
                }
                records.push(SocketRecord {
                    port: udp.local_port,
                    protocol: "UDP".to_string(),
                    status: "LISTEN".to_string(),
                    address: format_listen_address(&udp.local_addr.to_string(), udp.local_port),
                    pid,
                });
            }
        }
    }

    Ok(records)
}

fn format_listen_address(address: &str, port: u16) -> String {
    if address.contains(':') && !address.starts_with('[') {
        format!("[{address}]:{port}")
    } else {
        format!("{address}:{port}")
    }
}
