use windows::Wdk::System::SystemServices::RtlGetVersion;
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::System::SystemInformation::{
    GetNativeSystemInfo, OSVERSIONINFOEXW, SYSTEM_INFO,
};
use windows::core::Error;

pub struct SystemInfo {
    pub arch: &'static str,
    pub product_type: &'static str,
    pub version: String,
}

impl SystemInfo {
    pub fn collect() -> windows::core::Result<Self> {
        let (product_type, version) = Self::os()?;

        let arch = match Architecture::get() {
            Architecture::AMD64 => "AMD64",
            Architecture::ARM64 => "ARM64",
            Architecture::Unknown => "Unknown",
        };

        Ok(SystemInfo {
            arch,
            product_type,
            version,
        })
    }

    fn os() -> windows::core::Result<(&'static str, String)> {
        unsafe {
            let mut os_version = std::mem::MaybeUninit::<OSVERSIONINFOEXW>::zeroed();
            let os_version_ptr = os_version.as_mut_ptr();
            (*os_version_ptr).dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOEXW>() as u32;

            let ntstatus = RtlGetVersion(os_version_ptr as *mut _);

            if ntstatus == STATUS_SUCCESS {
                let os_version = os_version.assume_init();

                let product_type = match os_version.wProductType as u32 {
                    windows::Win32::System::SystemServices::VER_NT_WORKSTATION => "Workstation",
                    windows::Win32::System::SystemServices::VER_NT_SERVER => "Server",
                    windows::Win32::System::SystemServices::VER_NT_DOMAIN_CONTROLLER => {
                        "Domain Controller"
                    }
                    _ => "Unknown",
                };

                let version = format!(
                    "{}.{}.{}",
                    os_version.dwMajorVersion, os_version.dwMinorVersion, os_version.dwBuildNumber
                );

                Ok((product_type, version))
            } else {
                Err(Error::from_win32())
            }
        }
    }
}

enum Architecture {
    AMD64,
    ARM64,
    Unknown,
}

impl Architecture {
    fn get() -> Self {
        unsafe {
            let mut sysinfo = std::mem::MaybeUninit::<SYSTEM_INFO>::zeroed();
            GetNativeSystemInfo(sysinfo.as_mut_ptr());

            let sysinfo = sysinfo.assume_init();
            match sysinfo.Anonymous.Anonymous.wProcessorArchitecture.0 {
                9 => Architecture::AMD64,
                12 => Architecture::ARM64,
                _ => Architecture::Unknown,
            }
        }
    }
}
