use std::{os::raw::c_void, ptr};
use tracing::{error, info, warn};
use windows::Win32::{
    System::{
        Com::{CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx},
        ProcessStatus::GetModuleFileNameExW,
        Threading::{
            OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE, PROCESS_VM_READ,
            TerminateProcess,
        },
    },
    UI::{
        Shell::IShellDispatch,
        WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
    },
};
use windows::core::GUID;

const PROTECTED_PROCESSES: &[&str] = &[
    "Rainmeter.exe",
    "explorer.exe",
    "dwm.exe",
    "sihost.exe",
    "ShellExperienceHost.exe",
    "StartMenuExperienceHost.exe",
    "SearchHost.exe",
    "SearchUI.exe",
    "winlogon.exe",
    "lsass.exe",
    "csrss.exe",
    "svchost.exe",
    "services.exe",
    "smss.exe",
    "wininit.exe",
    "taskhostw.exe",
    "RuntimeBroker.exe",
    "ApplicationFrameHost.exe",
    "SystemSettings.exe",
    "force_quit_tool.exe",
];

const CLSID_SHELL_APPLICATION: GUID = GUID::from_values(
    0x13709620,
    0xC279,
    0x11CE,
    [0xA4, 0x9E, 0x44, 0x45, 0x53, 0x54, 0x00, 0x00],
);

fn extract_exe_name(full_path: &str) -> &str {
    full_path
        .trim_end_matches('\0')
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(full_path)
}

fn is_protected(exe_name: &str) -> bool {
    let lower = exe_name.to_lowercase();
    PROTECTED_PROCESSES
        .iter()
        .any(|&p| lower == p.to_lowercase())
}

pub fn trigger_shutdown_dialog() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let result: windows::core::Result<IShellDispatch> =
            CoCreateInstance(&CLSID_SHELL_APPLICATION, None, CLSCTX_INPROC_SERVER);

        match result {
            Ok(dispatch) => {
                let _ = dispatch.ShutdownWindows();
                info!("Shutdown dialog triggered via native COM");
            }
            Err(e) => {
                error!("Failed to invoke native shutdown dialog: {e}");
            }
        }
    }
}

pub fn force_quit() {
    unsafe {
        let current_window = GetForegroundWindow();

        let null: *mut c_void = ptr::null_mut();
        if current_window.0 == null {
            warn!("No foreground window found.");
            return;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(current_window, Some(&mut pid));

        if pid == 0 {
            warn!("Could not resolve PID for foreground window.");
            return;
        }

        let process = match OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_TERMINATE,
            false,
            pid,
        )
        .ok()
        {
            Some(h) => h,
            None => {
                error!("OpenProcess failed for PID {}.", pid);
                return;
            }
        };

        let mut name_buf = [0u16; 260];
        GetModuleFileNameExW(Some(process), None, &mut name_buf);

        let full_path = String::from_utf16_lossy(&name_buf);
        let exe_name = extract_exe_name(&full_path);

        if exe_name.is_empty() {
            error!("Could not determine exe name for PID {}.", pid);
            return;
        }

        if exe_name.eq_ignore_ascii_case("Code.exe") {
            let status = std::process::Command::new("wsl").arg("--shutdown").status();
            match status {
                Ok(s) if s.success() => info!("WSL shutdown initiated successfully."),
                Ok(s) => error!("WSL shutdown failed with code: {}", s),
                Err(e) => error!("Failed to execute WSL command: {}", e),
            }
        }

        if exe_name.eq_ignore_ascii_case("explorer.exe") {
            info!("Desktop focused, Triggering shutdown dialogue");
            trigger_shutdown_dialog();
            return;
        }

        if is_protected(exe_name) {
            warn!("Skipping protected process: {}", exe_name);
            return;
        }

        match TerminateProcess(process, 1) {
            Ok(()) => info!("Terminated: {} (PID {})", exe_name, pid),
            Err(e) => warn!("TerminateProcess failed for {}: {}", exe_name, e),
        }
    }
}
