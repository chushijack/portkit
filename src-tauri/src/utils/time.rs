//! 文件名称：time.rs
//!
//! 文件功能：
//! 提供占用时长与本地时间格式化。
//!
//! 主要职责：
//! - 把秒数格式化为 HH:MM:SS
//! - 把 Unix 时间戳格式化为详情页时间；0 表示未知，不能当成 1970-01-01
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use chrono::{Local, TimeZone};

/// 将占用秒数格式化为 `HH:MM:SS`，与原型图占用时间列一致。
fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{hours:02}:{minutes:02}:{secs:02}")
}

/// 将进程启动时间（Unix 秒）格式化为本地时间。
///
/// `0` 表示没读到启动时间，不能当成 1970-01-01。
pub fn format_local_datetime(unix_seconds: u64) -> String {
    if unix_seconds == 0 {
        return "-".to_string();
    }
    match Local.timestamp_opt(unix_seconds as i64, 0).single() {
        Some(datetime) => datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => "-".to_string(),
    }
}

/// 未知启动时间不展示从 Epoch 起算的假时长。
pub fn format_uptime(start_time: u64, occupied_seconds: u64) -> String {
    if start_time == 0 {
        "-".to_string()
    } else {
        format_duration(occupied_seconds)
    }
}

/// 当前本地时间，用于历史记录。
pub fn now_local_datetime() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::{format_duration, format_local_datetime, format_uptime};

    #[test]
    fn formats_hour_minute_second() {
        assert_eq!(format_duration(0), "00:00:00");
        assert_eq!(format_duration(754), "00:12:34");
        assert_eq!(format_duration(3661), "01:01:01");
    }

    #[test]
    fn unknown_epoch_is_placeholder() {
        assert_eq!(format_local_datetime(0), "-");
        assert_eq!(format_uptime(0, 1_786_000_000), "-");
        assert_eq!(format_uptime(1_700_000_000, 61), "00:01:01");
    }
}
