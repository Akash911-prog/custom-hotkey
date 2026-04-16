use windows::Win32::{
    System::{
        ProcessStatus::GetModuleFileNameExW,
        Threading::{
            OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE, PROCESS_VM_READ,
            TerminateProcess,
        },
    },
    UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
};

pub fn force_quit() {
    unsafe {
        let current_window = GetForegroundWindow();

        let mut pid: u32 = 0;

        GetWindowThreadProcessId(current_window, Some(&mut pid));

        let process = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_TERMINATE,
            false,
            pid,
        )
        .ok();

        let mut name_buf = [0u16; 260];
        GetModuleFileNameExW(process, None, &mut name_buf);
        let exe_name = String::from_utf16_lossy(&name_buf);

        if let Some(process) = process {
            if exe_name.contains("Code.exe") {
                let status = std::process::Command::new("wsl").arg("--shutdown").status();

                match status {
                    Ok(s) if s.success() => println!("WSL shutdown initiated successfully."),
                    Ok(s) => eprintln!("WSL shutdown failed with code: {}", s),
                    Err(e) => eprintln!("Failed to execute WSL command: {}", e),
                }
            }
            match TerminateProcess(process, 1) {
                Ok(()) => println!("closed"),
                Err(err) => println!("{}", err),
            };
        }
    };
}
