//! 文件名称：service_detector.rs
//!
//! 文件功能：
//! 根据进程名、端口和命令行识别常见开发服务。
//!
//! 主要职责：
//! - 给端口列表和快速搜索提供服务标签
//! - 给进程详情提供可读运行时名称
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

/// 识别开发服务展示名，例如 Vite Dev Server。
pub fn detect_service_label(process_name: &str, port: u16, command: &str) -> Option<String> {
    let name = process_name.to_lowercase();
    let cmd = command.to_lowercase();

    if is_node(&name, &cmd) {
        if cmd.contains("next") {
            return Some("Next.js".to_string());
        }
        if cmd.contains("vite") || port == 5173 || port == 5174 {
            return Some("Vite Dev Server".to_string());
        }
        if cmd.contains("vue") || cmd.contains("nuxt") {
            return Some("Vue Dev Server".to_string());
        }
        if cmd.contains("react") || port == 3000 || port == 3001 {
            return Some("Node.js".to_string());
        }
        return Some("Node.js".to_string());
    }

    if name.contains("python") || cmd.contains("python") {
        return Some("Python".to_string());
    }
    if name.contains("java") || name.contains("tomcat") {
        return Some("Java".to_string());
    }
    if name.contains("docker") || cmd.contains("docker") {
        return Some("Docker".to_string());
    }
    if name.contains("cargo") || name.contains("rustc") || cmd.contains("target/debug") {
        return Some("Rust".to_string());
    }
    if name.contains("postgres") || name.contains("psql") {
        return Some("PostgreSQL".to_string());
    }
    if name.contains("nginx") {
        return Some("Nginx".to_string());
    }

    None
}

/// 按占用端口去重后的服务标签，供详情页展示。
pub fn detect_service_labels(process_name: &str, ports: &[u16], command: &str) -> Vec<String> {
    let mut labels = Vec::new();
    let ports = if ports.is_empty() {
        &[0]
    } else {
        ports
    };
    for port in ports {
        let Some(label) = detect_service_label(process_name, *port, command) else {
            continue;
        };
        if !labels.iter().any(|item| item == &label) {
            labels.push(label);
        }
    }
    labels
}

/// 进程详情顶部的运行时描述。
pub fn detect_runtime_display_name(process_name: &str, command: &str) -> String {
    let name = process_name.to_lowercase();
    let cmd = command.to_lowercase();
    if is_node(&name, &cmd) {
        "Node.js JavaScript Runtime".to_string()
    } else if name.contains("python") {
        "Python Runtime".to_string()
    } else if name.contains("java") {
        "Java Runtime".to_string()
    } else if name.contains("docker") {
        "Docker".to_string()
    } else if name.contains("postgres") {
        "PostgreSQL".to_string()
    } else if name.contains("nginx") {
        "Nginx".to_string()
    } else {
        process_name.to_string()
    }
}

fn is_node(name: &str, command: &str) -> bool {
    name.contains("node") || command.contains("node")
}

#[cfg(test)]
mod tests {
    use super::detect_service_label;

    #[test]
    fn detects_vite_by_port() {
        let label = detect_service_label("node", 5173, "node ./node_modules/vite/bin/vite.js");
        assert_eq!(label.as_deref(), Some("Vite Dev Server"));
    }

    #[test]
    fn detects_python() {
        let label = detect_service_label("python", 8000, "python -m http.server");
        assert_eq!(label.as_deref(), Some("Python"));
    }

    #[test]
    fn unique_labels_keep_vite_and_skip_duplicates() {
        let labels = super::detect_service_labels(
            "node",
            &[1420, 5173],
            "node ./node_modules/vite/bin/vite.js",
        );
        assert_eq!(labels, vec!["Vite Dev Server"]);
    }
}
