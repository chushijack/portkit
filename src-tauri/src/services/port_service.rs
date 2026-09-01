//! 文件名称：port_service.rs
//!
//! 文件功能：
//! 提供端口扫描核心业务逻辑。
//!
//! 主要职责：
//! - 按范围过滤系统监听端口
//! - 组装进程名、占用时长与开发服务标签
//! - 按进程启动时间由近到远排列列表
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use std::cmp::Ordering;

use crate::config::is_common_port;
use crate::errors::AppError;
use crate::models::{PortInfo, ScanMode, ScanRequest};
use crate::services::service_detector;
use crate::system::{self, SocketRecord};
use crate::utils::format_uptime;

/// 扫描当前系统监听端口。
///
/// 返回：端口号、协议、状态、监听地址、占用进程与时长。
pub fn scan_ports(request: &ScanRequest) -> Result<Vec<PortInfo>, AppError> {
    let (start, end, common_only) = resolve_range(request)?;
    let sockets = system::list_sockets()?;
    let pids: Vec<u32> = sockets.iter().map(|item| item.pid).collect();
    let processes = system::query_processes(&pids);

    let mut ports = Vec::new();
    for socket in sockets {
        if !in_range(socket.port, start, end, common_only) {
            continue;
        }
        ports.push(to_port_info(socket, &processes));
    }

    ports.sort_by(|left, right| {
        left.port
            .cmp(&right.port)
            .then(left.protocol.cmp(&right.protocol))
            .then(left.pid.cmp(&right.pid))
    });
    ports.dedup_by(|left, right| {
        left.port == right.port && left.protocol == right.protocol && left.pid == right.pid
    });
    // 占用越短说明启动越近；先按端口去重，再按启动时间由近到远。
    ports.sort_by(cmp_recent_start);
    Ok(ports)
}

/// 启动时间由近到远；读不到启动时间的排到最后。
fn cmp_recent_start(left: &PortInfo, right: &PortInfo) -> Ordering {
    match (left.start_time == 0, right.start_time == 0) {
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        _ => right
            .start_time
            .cmp(&left.start_time)
            .then(left.port.cmp(&right.port))
            .then(left.protocol.cmp(&right.protocol))
            .then(left.pid.cmp(&right.pid)),
    }
}

fn resolve_range(request: &ScanRequest) -> Result<(u16, u16, bool), AppError> {
    match request.mode {
        ScanMode::All => Ok((1, 65535, false)),
        ScanMode::Common => Ok((1, 65535, true)),
        ScanMode::Custom => {
            let start = request.start_port.unwrap_or(1);
            let end = request.end_port.unwrap_or(65535);
            if start == 0 || end == 0 || start > end {
                return Err(AppError::InvalidScanRange);
            }
            Ok((start, end, false))
        }
    }
}

fn in_range(port: u16, start: u16, end: u16, common_only: bool) -> bool {
    if common_only {
        return is_common_port(port);
    }
    port >= start && port <= end
}

fn to_port_info(
    socket: SocketRecord,
    processes: &std::collections::HashMap<u32, system::ProcessSnapshot>,
) -> PortInfo {
    let (process_name, start_time, occupied_seconds, command) = match processes.get(&socket.pid) {
        Some(process) => (
            process.name.clone(),
            process.start_time,
            process.occupied_seconds,
            Some(process.command.clone()).filter(|item| !item.is_empty()),
        ),
        None if socket.pid == 0 => ("System".to_string(), 0, 0, None),
        None => ("未知进程".to_string(), 0, 0, None),
    };
    let service_label = service_detector::detect_service_label(
        &process_name,
        socket.port,
        command.as_deref().unwrap_or(""),
    );

    PortInfo {
        port: socket.port,
        protocol: socket.protocol,
        status: socket.status,
        listen_address: socket.address,
        process_name,
        pid: socket.pid,
        occupied_time: format_uptime(start_time, occupied_seconds),
        occupied_seconds,
        start_time,
        service_label,
        command,
    }
}

#[cfg(test)]
mod tests {
    use super::{cmp_recent_start, in_range, resolve_range};
    use crate::models::{PortInfo, ScanMode, ScanRequest};

    fn sample_port(port: u16, start_time: u64) -> PortInfo {
        PortInfo {
            port,
            protocol: "TCP".into(),
            status: "LISTEN".into(),
            listen_address: format!("127.0.0.1:{port}"),
            process_name: "node".into(),
            pid: u32::from(port),
            occupied_time: String::new(),
            occupied_seconds: 0,
            start_time,
            service_label: None,
            command: None,
        }
    }

    #[test]
    fn lists_recently_started_processes_first() {
        let newer = sample_port(5173, 1_700_000_090);
        let older = sample_port(53, 1_700_000_000);
        assert_eq!(cmp_recent_start(&newer, &older), std::cmp::Ordering::Less);
        assert_eq!(cmp_recent_start(&older, &newer), std::cmp::Ordering::Greater);
    }

    #[test]
    fn unknown_start_time_sorts_last() {
        let known = sample_port(5173, 1_700_000_000);
        let unknown = sample_port(5353, 0);
        assert_eq!(cmp_recent_start(&known, &unknown), std::cmp::Ordering::Less);
        assert_eq!(cmp_recent_start(&unknown, &known), std::cmp::Ordering::Greater);
    }

    #[test]
    fn custom_range_rejects_inverted_bounds() {
        let request = ScanRequest {
            mode: ScanMode::Custom,
            start_port: Some(8080),
            end_port: Some(80),
            record_history: false,
        };
        assert!(resolve_range(&request).is_err());
    }

    #[test]
    fn common_mode_only_allows_preset_ports() {
        assert!(in_range(5173, 1, 65535, true));
        assert!(in_range(3306, 1, 65535, true));
        assert!(!in_range(1234, 1, 65535, true));
    }
}
