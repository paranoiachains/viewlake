use windows::Wdk::System::SystemServices::RtlGetVersion;
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::System::SystemInformation::{GetNativeSystemInfo, OSVERSIONINFOW, SYSTEM_INFO};

pub enum Architecture {
    AMD64,
    ARM64,
    Unknown,
}

pub fn get_arch() -> Architecture {
    unsafe {
        let mut sysinfo: SYSTEM_INFO = std::mem::zeroed();
        GetNativeSystemInfo(&mut sysinfo as *mut _);

        let arch = sysinfo.Anonymous.Anonymous.wProcessorArchitecture;
        match arch.0 {
            9 => Architecture::AMD64,
            12 => Architecture::ARM64,
            _ => Architecture::Unknown,
        }
    }
}

pub fn get_os() -> Option<OSVERSIONINFOW> {
    unsafe {
        let mut os_version: OSVERSIONINFOW = std::mem::zeroed();
        let ntstatus = RtlGetVersion(&mut os_version as *mut _);
        if ntstatus == STATUS_SUCCESS {
            Some(os_version)
        } else {
            None
        }
    }
}
