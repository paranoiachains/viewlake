use core::fmt;
use windows::Wdk::System::SystemServices::*;
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::System::SystemInformation::*;
use windows::Win32::System::SystemServices::*;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct OsInfo {
    product_type: ProductType,
    version: WindowsVersion,
}

impl OsInfo {
    pub fn new(product_type: ProductType, version: WindowsVersion) -> Self {
        OsInfo {
            product_type,
            version,
        }
    }
}

#[derive(Debug)]
pub enum ProductType {
    Workstation,
    Server,
    DomainController,
    Unknown,
}

#[derive(Debug)]
pub struct WindowsVersion {
    pub major: u32,
    pub minor: u32,
    pub build: u32,
}

impl WindowsVersion {
    pub fn new(major: u32, minor: u32, build: u32) -> Self {
        Self {
            major,
            minor,
            build,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.build)
    }
}

impl fmt::Display for WindowsVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.build)
    }
}

pub fn get_os() -> Option<OsInfo> {
    unsafe {
        let mut os_version: OSVERSIONINFOEXW = std::mem::zeroed();
        let ntstatus = RtlGetVersion(&mut os_version as *mut _ as *mut OSVERSIONINFOW);
        if ntstatus == STATUS_SUCCESS {
            let product_type = match os_version.wProductType {
                VER_NT_WORKSTATION => ProductType::Workstation,
                VER_NT_SERVER => ProductType::Server,
                VER_NT_DOMAIN_CONTROLLER => ProductType::DomainController,
                _ => ProductType::Unknown,
            };

            let version = WindowsVersion::new(
                os_version.dwMajorVersion,
                os_version.dwMinorVersion,
                os_version.dwBuildNumber,
            );

            Some(OsInfo {
                product_type,
                version,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_system_info() {
        let arch = get_arch();
        println!("Arch: {:?}", arch);

        let os = get_os().expect("Failed to get OS info");
        println!("OS Version: {}", os.version);
        println!("Product Type: {:?}", os.product_type);

        assert!(os.version.major >= 6);
    }
}
