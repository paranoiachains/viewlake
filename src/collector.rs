use std::ptr::null_mut;

use windows::Wdk::System::SystemServices::*;
use windows::Win32::Foundation::NO_ERROR;
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::NetworkManagement::IpHelper::GAA_FLAG_INCLUDE_PREFIX;
use windows::Win32::NetworkManagement::IpHelper::GetAdaptersAddresses;
use windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH;
use windows::Win32::NetworkManagement::NetManagement::*;
use windows::Win32::Networking::WinSock::AF_UNSPEC;
use windows::Win32::System::SystemInformation::*;
use windows::Win32::System::SystemServices::*;
use windows::core::Error;
use windows::core::PCWSTR;
use windows::core::PWSTR;

// This logic implies only logging/collecting, not pattern matching

#[derive(Debug)]
pub struct SystemInfo {
    pub arch: String,
    pub product_type: String,
    pub version: String,
}

impl SystemInfo {
    pub fn collect() -> Result<Self, Error> {
        let (product_type, version) = Self::os()?;

        let arch = match Architecture::get() {
            Architecture::AMD64 => "AMD64".to_string(),
            Architecture::ARM64 => "ARM64".to_string(),
            Architecture::Unknown => "Unknown".to_string(),
        };

        Ok(SystemInfo {
            arch,
            product_type,
            version,
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

pub struct NetworkInfo {
    fqdn: String,
    domain_or_workgroup: String,
    status: String,
    network: Vec<String>,
}

impl NetworkInfo {
    pub fn collect() -> Result<NetworkInfo, Error> {
        let (domain_or_workgroup, status) = Self::domain_or_workgroup()?;
        Ok(NetworkInfo {
            fqdn: Self::fqdn()?,
            domain_or_workgroup,
            status,
            network: Self::network()?,
        })
    }
    fn fqdn() -> Result<String, Error> {
        unsafe {
            let mut size: u32 = 512;
            GetComputerNameExW(
                ComputerNameDnsFullyQualified,
                PWSTR::null(),
                size as *mut u32,
            )?;

            println!("Buffer size: {}", size);

            let mut buffer: Vec<u16> = vec![0; size as usize];

            GetComputerNameExW(
                ComputerNameDnsFullyQualified,
                PWSTR(buffer.as_mut_ptr()),
                &mut size as *mut u32,
            )?;

            buffer.truncate(size as usize);

            Ok(String::from_utf16_lossy(&buffer))
        }
    }

    fn domain_or_workgroup() -> Result<(String, String), Error> {
        unsafe {
            let mut buffer = PWSTR::null();
            let mut status_result = NETSETUP_JOIN_STATUS::default();
            if NetGetJoinInformation(
                PCWSTR::null(),
                &mut buffer as *mut _,
                &mut status_result as *mut _,
            ) != NERR_Success
            {
                return Err(Error::from_win32());
            }

            let domain_or_workgroup = if !buffer.is_null() {
                let mut len = 0;
                while *buffer.0.add(len) != 0 {
                    len += 1;
                }
                let slice = std::slice::from_raw_parts(buffer.0, len);
                String::from_utf16_lossy(slice)
            } else {
                String::new()
            };

            NetApiBufferFree(Some(buffer.0 as _));

            let status = match status_result {
                NetSetupUnjoined => "Unjoined".to_string(),
                NetSetupWorkgroupName => "Workgroup".to_string(),
                NetSetupDomainName => "Domain".to_string(),
                _ => "Unknown".to_string(),
            };

            Ok((domain_or_workgroup, status))
        }
    }

    fn network() -> Result<Vec<String>, Error> {
        unsafe {
            let mut size = 0u32;

            GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                GAA_FLAG_INCLUDE_PREFIX,
                None,
                None,
                &mut size as *mut _,
            );

            let mut buffer = vec![0u8; size as usize];
            let adapter_addresess = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

            let ret = GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                GAA_FLAG_INCLUDE_PREFIX,
                None,
                Some(adapter_addresess),
                &mut size as *mut _,
            );

            if ret != 0 {
                return Err(Error::from_win32());
            }

            let mut current = adapter_addresess;
            let mut adapter_vec: Vec<String> = Vec::new();
            while !current.is_null() {
                let adapter = &*current;
                let name = if !adapter.FriendlyName.is_null() {
                    adapter
                        .FriendlyName
                        .to_string()
                        .expect("Error while converting friendly name to string")
                } else {
                    String::new()
                };

                adapter_vec.push(name);

                current = adapter.Next;
            }

            Ok(adapter_vec)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_system_info() {
        let os_info = SystemInfo::collect().expect("Failed to get OS info");

        println!("Arch: {}", os_info.arch);
        println!("OS Version: {}", os_info.version);
        println!("Product Type: {}", os_info.product_type);

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
    }

    #[test]
    fn get_network_info() {
        let nwinfo = NetworkInfo::collect().expect("Failed to collect network info");
        println!("fqdn: {}", nwinfo.fqdn);
    }
}
