use windows::Wdk::System::SystemServices::*;
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::System::SystemInformation::*;
use windows::Win32::System::SystemServices::*;
use windows::core::Error;
use windows::core::PWSTR;

// This logic implies only logging/collecting, not pattern matching

#[derive(Debug)]
pub struct SystemInfo {
    pub arch: String,
    pub product_type: String,
    pub version: String,
    pub fqdn: String,
}

impl SystemInfo {
    pub fn collect() -> Result<Self, Error> {
        let (product_type, version) = Self::os()?;
        let fqdn = Self::os()?;

        let arch = match Architecture::get() {
            Architecture::AMD64 => "AMD64".to_string(),
            Architecture::ARM64 => "ARM64".to_string(),
            Architecture::Unknown => "Unknown".to_string(),
        };

        Ok(SystemInfo {
            arch,
            product_type,
            version,
            fqdn,
        })
    }

    fn os() -> Result<(String, String), Error> {
        unsafe {
            let mut os_version: OSVERSIONINFOEXW = std::mem::zeroed();
            let ntstatus = RtlGetVersion(&mut os_version as *mut _ as *mut OSVERSIONINFOW);

            if ntstatus == STATUS_SUCCESS {
                let product_type = match os_version.wProductType as u32 {
                    VER_NT_WORKSTATION => "Workstation".to_string(),
                    VER_NT_SERVER => "Server".to_string(),
                    VER_NT_DOMAIN_CONTROLLER => "Domain Controller".to_string(),
                    _ => "Unknown".to_string(),
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

    fn fqdn() -> Result<String, Error> {
        unsafe {
            let mut size: u32 = 0;
            GetComputerNameExW(
                ComputerNameDnsFullyQualified,
                PWSTR::null(),
                &mut size as *mut u32,
            )?;

            let mut buffer: Vec<u16> = vec![0; size as usize];

            GetComputerNameExW(
                ComputerNameDnsFullyQualified,
                PWSTR(buffer.as_mut_ptr()),
                &mut size as *mut u32,
            )?;

            buffer.truncate(size as usize);

            Ok(String::from_utf16_lossy(&buffer));
        }
    }
}

#[derive(Debug)]
enum Architecture {
    AMD64,
    ARM64,
    Unknown,
}

impl Architecture {
    fn get() -> Self {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_system_info() {
        let os_info = SystemInfo::collect().expect("Failed to get OS info");

        println!("Arch: {:?}", os_info.arch);
        println!("OS Version: {}", os_info.version);
        println!("Product Type: {:?}", os_info.product_type);

        assert!(
            os_info.arch == "AMD64" || os_info.arch == "ARM64" || os_info.arch == "Unknown",
            "Unexpected architecture: {}",
            os_info.arch
        );

        assert!(
            os_info.version.chars().next().unwrap().is_ascii_digit(),
            "OS version is not a valid number: {}",
            os_info.version
        );

        assert!(
            os_info.product_type == "Workstation"
                || os_info.product_type == "Server"
                || os_info.product_type == "Domain Controller"
                || os_info.product_type == "Unknown",
            "Unexpected product type: {}",
            os_info.product_type
        );

        assert!(!os_info.fqdn.is_empty(), "FQDN is empty");
    }
}
