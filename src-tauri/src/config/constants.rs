//! 文件名称：constants.rs
//!
//! 文件功能：
//! 存放扫描默认值与危险进程名单。
//!
//! 主要职责：
//! - 定义常用开发端口
//! - 定义禁止结束的系统进程
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

/// 开发场景常用端口，对应快速扫描预设与底栏菜单。
pub struct CommonPortDef {
    /// 监听端口号。
    pub port: u16,
    /// 常见软件备注，展示在端口号后面。
    pub label: &'static str,
}

/// 端口按数字排列；备注写常见软件，方便对照。
pub const COMMON_PORT_ITEMS: &[CommonPortDef] = &[
    CommonPortDef { port: 22, label: "SSH" },
    CommonPortDef { port: 80, label: "HTTP" },
    CommonPortDef { port: 443, label: "HTTPS" },
    CommonPortDef { port: 1433, label: "SQL Server" },
    CommonPortDef { port: 2181, label: "ZooKeeper" },
    CommonPortDef { port: 3000, label: "React / Next.js" },
    CommonPortDef { port: 3001, label: "React (alt)" },
    CommonPortDef { port: 3030, label: "Warp" },
    CommonPortDef { port: 3306, label: "MySQL" },
    CommonPortDef { port: 4000, label: "Phoenix / Jekyll" },
    CommonPortDef { port: 4173, label: "Vite Preview" },
    CommonPortDef { port: 4200, label: "Angular" },
    CommonPortDef { port: 5000, label: "Flask / Rails" },
    CommonPortDef { port: 5005, label: "Java Debug" },
    CommonPortDef { port: 5173, label: "Vite" },
    CommonPortDef { port: 5174, label: "Vite (alt)" },
    CommonPortDef { port: 5432, label: "PostgreSQL" },
    CommonPortDef { port: 5672, label: "RabbitMQ" },
    CommonPortDef { port: 6060, label: "Go pprof" },
    CommonPortDef { port: 6379, label: "Redis" },
    CommonPortDef { port: 8000, label: "Django / Axum" },
    CommonPortDef { port: 8080, label: "Spring / Go / Vue" },
    CommonPortDef { port: 8081, label: "Vue (alt)" },
    CommonPortDef { port: 8443, label: "Spring HTTPS" },
    CommonPortDef { port: 8888, label: "Jupyter" },
    CommonPortDef { port: 9000, label: "Sonar / Dev HTTP" },
    CommonPortDef { port: 9090, label: "Prometheus" },
    CommonPortDef { port: 9092, label: "Kafka" },
    CommonPortDef { port: 9200, label: "Elasticsearch" },
    CommonPortDef { port: 11211, label: "Memcached" },
    CommonPortDef { port: 11434, label: "Ollama" },
    CommonPortDef { port: 27017, label: "MongoDB" },
];

/// 是否属于常用开发端口。
pub fn is_common_port(port: u16) -> bool {
    COMMON_PORT_ITEMS.iter().any(|item| item.port == port)
}

/// 结束这些进程会破坏系统会话，服务层必须拦截。
pub const PROTECTED_PROCESS_NAMES: &[&str] = &[
    "system",
    "registry",
    "smss",
    "csrss",
    "wininit",
    "services",
    "lsass",
    "winlogon",
    "svchost",
    "dwm",
    "fontdrvhost",
    "lsm",
    "idle",
    "memory compression",
    "secure system",
    "kernel_task",
    "launchd",
    "windowserver",
    "loginwindow",
    "cfprefsd",
    "opendirectoryd",
];

#[cfg(test)]
mod tests {
    use super::{is_common_port, COMMON_PORT_ITEMS};
    use std::collections::HashSet;

    #[test]
    fn common_ports_are_unique() {
        let mut seen = HashSet::new();
        for item in COMMON_PORT_ITEMS {
            assert!(seen.insert(item.port), "duplicate common port {}", item.port);
        }
    }

    #[test]
    fn covers_requested_dev_stacks() {
        assert!(is_common_port(3000));
        assert!(is_common_port(3030));
        assert!(is_common_port(3306));
        assert!(is_common_port(5005));
        assert!(is_common_port(5173));
        assert!(is_common_port(6060));
        assert!(is_common_port(6379));
        assert!(is_common_port(8000));
        assert!(is_common_port(8080));
        assert!(!is_common_port(1234));
    }
}
