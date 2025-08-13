use std::ptr::null_mut;

use windows::Wdk::System::SystemServices::*;
use windows::Win32::Foundation::NO_ERROR;
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::NetworkManagement::IpHelper::GAA_FLAG_INCLUDE_PREFIX;
use windows::Win32::NetworkManagement::IpHelper::GetAdaptersAddresses;
use windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH;
use windows::Win32::NetworkManagement::NetManagement::*;
use windows::Win32::Networking::WinSock::*;
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
    hostname: String,
    domain_or_workgroup: String,
    status: String,
    adapters_info: Vec<Adapter>,
}

#[derive(Debug)]
pub struct Adapter {
    friendly_name: String,
    description: String,
    ipv4_addresses: Option<Vec<String>>,
    gateways: Option<String>,
}

impl Adapter {
    pub fn new(
        friendly_name: String,
        description: String,
        ipv4_addresses: Option<Vec<String>>,
        gateways: Option<String>,
    ) -> Self {
        Adapter {
            friendly_name,
            description,
            ipv4_addresses,
            gateways,
        }
    }
}

impl NetworkInfo {
    pub fn collect() -> Result<NetworkInfo, Error> {
        let (domain_or_workgroup, status) = Self::domain_or_workgroup()?;
        Ok(NetworkInfo {
            hostname: Self::hostname()?,
            domain_or_workgroup,
            status,
            adapters_info: Self::adapters_info()?,
        })
    }
    fn hostname() -> Result<String, Error> {
        unsafe {
            let mut size: u32 = 512;
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
                NetSetupWorkgroupName => "Joined to Workgroup".to_string(),
                NetSetupDomainName => "Joined to Domain".to_string(),
                _ => "Unknown".to_string(),
            };

            Ok((domain_or_workgroup, status))
        }
    }

    fn adapters_info() -> Result<Vec<Adapter>, Error> {
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
            let mut adapter_vec: Vec<Adapter> = Vec::new();
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

                let description = if !adapter.Description.is_null() {
                    adapter
                        .Description
                        .to_string()
                        .expect("Error while converting desc to string")
                } else {
                    String::new()
                };

                let ipv4_addrs = Self::get_ipv4_addresses(adapter);
                let gateway = Self::get_default_gateway(adapter);

                let instance = Adapter::new(name, description, ipv4_addrs, gateway);
                adapter_vec.push(instance);
            }
            Ok(adapter_vec)
        }
    }

    fn get_ipv4_addresses(adapter: &IP_ADAPTER_ADDRESSES_LH) -> Option<Vec<String>> {
        unsafe {
            let mut ipv4_addrs = Vec::new();
            let mut current_unicast = adapter.FirstUnicastAddress;

            while !current_unicast.is_null() {
                let unicast = &*current_unicast;
                let sockaddr = unicast.Address.lpSockaddr;

                if !sockaddr.is_null() {
                    let family = (*sockaddr).sa_family;
                    if family == AF_INET {
                        let ipv4 = *(sockaddr as *const SOCKADDR_IN);
                        let octets = ipv4.sin_addr.S_un.S_un_b;
                        ipv4_addrs.push(format!(
                            "{}.{}.{}.{}",
                            octets.s_b1, octets.s_b2, octets.s_b3, octets.s_b4
                        ));
                    }
                }

                current_unicast = unicast.Next;
            }
            if !ipv4_addrs.is_empty() {
                Some(ipv4_addrs)
            } else {
                None
            }
        }
    }

    fn get_default_gateway(adapter: &IP_ADAPTER_ADDRESSES_LH) -> Option<String> {
        unsafe {
            let mut current_gw = adapter.FirstGatewayAddress;

            while !current_gw.is_null() {
                let gw = &*current_gw;
                let sockaddr = gw.Address.lpSockaddr;

                if !sockaddr.is_null() {
                    let family = (*sockaddr).sa_family;
                    if family == AF_INET {
                        let ipv4 = *(sockaddr as *const SOCKADDR_IN);
                        let octets = ipv4.sin_addr.S_un.S_un_b;
                        return Some(format!(
                            "{}.{}.{}.{}",
                            octets.s_b1, octets.s_b2, octets.s_b3, octets.s_b4
                        ));
                    }
                }

                current_gw = gw.Next;
            }
        }
        None
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
        println!("hostname: {}", nwinfo.hostname);
        println!("domain_or_workgroup: {}", nwinfo.domain_or_workgroup);
        println!("status: {}", nwinfo.status);
        println!("adapters: {:?}", nwinfo.adapters_info);
    }
}
