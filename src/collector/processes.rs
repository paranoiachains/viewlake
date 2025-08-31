use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::*;
use windows::Win32::System::ProcessStatus::{
    EnumProcessModules, EnumProcesses, GetModuleBaseNameW,
};
use windows::Win32::System::Threading::*;

#[derive(Debug)]
struct ProcessList {
    processes: Vec<Process>,
}

#[derive(Debug)]
struct Process {
    pid: u32,
    name: Option<String>,
}

impl ProcessList {
    fn collect() -> Result<Self, windows::core::Error> {
        let pids = Self::get_pid_list()?;

        let mut processes: Vec<Process> = Vec::new();
        for pid in pids {
            let name = Self::process_name_from_pid(pid);

            let process = Process { pid, name };
            processes.push(process);
        }

        Ok(ProcessList { processes })
    }

    fn get_pid_list() -> Result<Vec<u32>, windows::core::Error> {
        let mut buf: Vec<u32> = vec![0; 1024];
        let mut bytes_returned: u32 = 0;

        unsafe {
            EnumProcesses(
                buf.as_mut_ptr(),
                (buf.len() * std::mem::size_of::<u32>()) as u32,
                &mut bytes_returned,
            )?;
        }

        let count = (bytes_returned as usize) / std::mem::size_of::<u32>();
        buf.truncate(count);

        Ok(buf)
    }

    fn process_name_from_pid(pid: u32) -> Option<String> {
        unsafe {
            let handle: HANDLE =
                OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid).unwrap();

            if handle.is_invalid() {
                return None;
            }

            let mut hmod = [0isize; 1024];
            let mut needed = 0u32;

            if EnumProcessModules(
                handle,
                hmod.as_mut_ptr() as *mut HMODULE,
                std::mem::size_of_val(&hmod) as u32,
                &mut needed,
            )
            .is_ok()
            {
                let mut buffer = [0u16; 260];
                if GetModuleBaseNameW(handle, HMODULE(hmod[0]), &mut buffer) > 0 {
                    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
                    let name = OsString::from_wide(&buffer[..len])
                        .to_string_lossy()
                        .into_owned();
                    return Some(name);
                }
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_processes() {
        let processes = ProcessList::collect();
        println!("{:?}", processes);

        assert!(processes.is_ok(), "ProcessList shouldn't be Err");
    }
}
