use serde::{Deserialize, Serialize};
use windows::Wdk::System::SystemServices::RtlGetVersion;
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::System::SystemInformation::{
    GetNativeSystemInfo, OSVERSIONINFOEXW, SYSTEM_INFO,
};
use windows::core::Error;

#[derive(Deserialize, Serialize)]
pub struct SystemInfo {
    pub arch: String,
    pub product_type: u8,
    pub version: Version,
}

#[derive(Deserialize, Serialize)]
pub struct Version {
    pub major_version: u32,
    pub minor_version: u32,
    pub build_number: u32,
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
            arch: arch.to_string(),
            product_type,
            version,
        })
    }

    fn os() -> windows::core::Result<(u8, Version)> {
        unsafe {
            let mut os_version = std::mem::MaybeUninit::<OSVERSIONINFOEXW>::zeroed();
            let os_version_ptr = os_version.as_mut_ptr();
            (*os_version_ptr).dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOEXW>() as u32;

            let ntstatus = RtlGetVersion(os_version_ptr as *mut _);

            if ntstatus == STATUS_SUCCESS {
                let os_version = os_version.assume_init();

                Ok((
                    os_version.wProductType,
                    Version {
                        major_version: os_version.dwMajorVersion,
                        minor_version: os_version.dwMinorVersion,
                        build_number: os_version.dwBuildNumber,
                    },
                ))
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

            match sysinfo
                .assume_init()
                .Anonymous
                .Anonymous
                .wProcessorArchitecture
                .0
            {
                9 => Architecture::AMD64,
                12 => Architecture::ARM64,
                _ => Architecture::Unknown,
            }
        }
    }
}
