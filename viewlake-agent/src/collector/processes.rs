use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::*;
use windows::Win32::System::ProcessStatus::*;
use windows::Win32::System::Threading::*;

#[derive(Deserialize, Serialize)]
pub struct ProcessList {
    pub list: Vec<Process>,
}

#[derive(Deserialize, Serialize)]
pub struct Process {
    pub pid: u32,
    pub name: String,
}

impl ProcessList {
    pub fn collect() -> Result<Self, windows::core::Error> {
        let pids = Self::get_pid_list()?;

        let mut processes = Vec::with_capacity(256);

        for pid in pids.iter().copied().filter(|&p| p != 0) {
            if let Some(name) = Self::process_name_from_pid(pid) {
                processes.push(Process { pid, name });
            }
        }

        Ok(Self { list: processes })
    }

    fn get_pid_list() -> Result<Vec<u32>, windows::core::Error> {
        let mut buf = vec![0u32; 1024];
        let mut bytes_returned = 0u32;

        unsafe {
            EnumProcesses(
                buf.as_mut_ptr(),
                (buf.len() * std::mem::size_of::<u32>()) as u32,
                &mut bytes_returned,
            )?;
        }

        buf.truncate((bytes_returned as usize) / std::mem::size_of::<u32>());
        Ok(buf)
    }

    fn process_name_from_pid(pid: u32) -> Option<String> {
        unsafe {
            let handle =
                OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid).ok()?;
            let mut hmod = [HMODULE(0); 1024];
            let mut needed = 0u32;

            if EnumProcessModules(
                handle,
                hmod.as_mut_ptr(),
                std::mem::size_of_val(&hmod) as u32,
                &mut needed,
            )
            .is_ok()
            {
                let mut buf = [0u16; 260];
                let len = GetModuleBaseNameW(handle, hmod[0], &mut buf) as usize;
                if len > 0 {
                    let name = String::from_utf16_lossy(&buf[..len]);
                    return Some(name);
                }
            }

            None
        }
    }
}
