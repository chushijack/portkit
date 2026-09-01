//! 文件名称：service.rs
//!
//! 文件功能：
//! 按 PID 查找并停止 Windows 服务，避免 SCM 把杀进程当成崩溃重启。
//!
//! 主要职责：
//! - EnumServicesStatusExW 把 PID 映射到服务名
//! - SERVICE_CONTROL_STOP 请求停止服务
//!
//! 作者：Chushi Jack
//! 创建时间：2026-08-20

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::System::Services::{
    CloseServiceHandle, ControlService, EnumServicesStatusExW, OpenSCManagerW, OpenServiceW,
    ENUM_SERVICE_STATUS_PROCESSW, SC_ENUM_PROCESS_INFO, SC_MANAGER_CONNECT,
    SC_MANAGER_ENUMERATE_SERVICE, SERVICE_CONTROL_STOP, SERVICE_QUERY_STATUS, SERVICE_STATE_ALL,
    SERVICE_STATUS, SERVICE_STOP, SERVICE_WIN32,
};

use crate::errors::AppError;

struct WinService {
    name: String,
    display: String,
}

/// 查找占用该 PID 的 Windows 服务。
fn service_by_pid(pid: u32) -> Option<WinService> {
    unsafe {
        let manager = OpenSCManagerW(
            PCWSTR::null(),
            PCWSTR::null(),
            SC_MANAGER_CONNECT | SC_MANAGER_ENUMERATE_SERVICE,
        )
        .ok()?;
        let found = find_service_by_pid(manager, pid);
        let _ = CloseServiceHandle(manager);
        found
    }
}

/// 查找占用该 PID 的服务名，供控制者展示。
pub fn service_name_by_pid(pid: u32) -> Option<String> {
    service_by_pid(pid).map(|item| {
        if item.name.is_empty() {
            item.display
        } else {
            item.name
        }
    })
}

/// 若该 PID 属于 Windows 服务，则请求停止服务。
///
/// 不是服务时返回 Ok(false)；已发出停止请求返回 Ok(true)。
pub fn stop_owning_service(pid: u32) -> Result<bool, AppError> {
    let Some(item) = service_by_pid(pid) else {
        return Ok(false);
    };
    stop_named_service(&item.name)?;
    Ok(true)
}

fn find_service_by_pid(
    manager: windows::Win32::System::Services::SC_HANDLE,
    pid: u32,
) -> Option<WinService> {
    unsafe {
        let mut bytes_needed = 0_u32;
        let mut returned = 0_u32;
        let mut resume = 0_u32;
        let _ = EnumServicesStatusExW(
            manager,
            SC_ENUM_PROCESS_INFO,
            SERVICE_WIN32,
            SERVICE_STATE_ALL,
            None,
            &mut bytes_needed,
            &mut returned,
            Some(&mut resume as *mut u32),
            PCWSTR::null(),
        );
        if bytes_needed == 0 {
            return None;
        }
        let mut buffer = vec![0_u8; bytes_needed as usize];
        resume = 0;
        EnumServicesStatusExW(
            manager,
            SC_ENUM_PROCESS_INFO,
            SERVICE_WIN32,
            SERVICE_STATE_ALL,
            Some(buffer.as_mut_slice()),
            &mut bytes_needed,
            &mut returned,
            Some(&mut resume as *mut u32),
            PCWSTR::null(),
        )
        .ok()?;
        let services = std::slice::from_raw_parts(
            buffer.as_ptr().cast::<ENUM_SERVICE_STATUS_PROCESSW>(),
            returned as usize,
        );
        services.iter().find_map(|item| {
            if item.ServiceStatusProcess.dwProcessId != pid {
                return None;
            }
            Some(WinService {
                name: pwstr_to_string(item.lpServiceName),
                display: pwstr_to_string(item.lpDisplayName),
            })
        })
    }
}

fn stop_named_service(name: &str) -> Result<(), AppError> {
    let mut wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let manager = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
            .map_err(map_service_error)?;
        let service = OpenServiceW(
            manager,
            PCWSTR(wide.as_mut_ptr()),
            SERVICE_STOP | SERVICE_QUERY_STATUS,
        );
        let result = match service {
            Ok(service) => {
                let mut status = SERVICE_STATUS::default();
                let ok = ControlService(service, SERVICE_CONTROL_STOP, &mut status);
                let _ = CloseServiceHandle(service);
                ok.map_err(map_service_error)
            }
            Err(error) => Err(map_service_error(error)),
        };
        let _ = CloseServiceHandle(manager);
        result
    }
}

fn map_service_error(error: windows::core::Error) -> AppError {
    let code = error.code().0 as u32;
    if code == 5 || code == 0x8007_0005 {
        AppError::PermissionDenied
    } else {
        AppError::SystemError(error.message())
    }
}

fn pwstr_to_string(value: PWSTR) -> String {
    if value.is_null() {
        return String::new();
    }
    unsafe { value.to_string().unwrap_or_default() }
}
