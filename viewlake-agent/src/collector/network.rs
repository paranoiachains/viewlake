use windows::Win32::NetworkManagement::IpHelper::*;
use windows::Win32::NetworkManagement::NetManagement::*;
use windows::Win32::Networking::WinSock::*;
use windows::Win32::System::SystemInformation::*;
use windows::core::{Error, PCWSTR, PWSTR};

pub struct NetworkInfo {
    pub hostname: Option<String>,
    pub domain_or_workgroup: String,
    pub status: String,
    pub adapters: Option<Vec<Adapter>>,
}

pub struct Adapter {
    pub friendly_name: String,
    pub description: String,
    pub ipv4_addresses: Option<Vec<String>>,
    pub gateways: Option<String>,
}

impl NetworkInfo {
    pub fn collect() -> Result<NetworkInfo, Error> {
        let (domain_or_workgroup, status) = Self::domain_or_workgroup()?;

        Ok(NetworkInfo {
            hostname: Self::hostname().ok(),
            domain_or_workgroup,
            status,
            adapters: Self::adapters_info().ok(),
        })
    }

    fn hostname() -> Result<String, Error> {
        unsafe {
            let mut size: u32 = 15000;
            let mut buffer: Vec<u16> = vec![0; size as usize];

            GetComputerNameExW(
                ComputerNameDnsFullyQualified,
                PWSTR(buffer.as_mut_ptr()),
                &mut size,
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

            // first call - get required buffer size
            let _ = GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                GAA_FLAG_INCLUDE_PREFIX,
                None,
                None,
                &mut size,
            );

            let mut buffer = vec![0u8; size as usize];
            let adapters_ptr = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

            let ret = GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                GAA_FLAG_INCLUDE_PREFIX | GAA_FLAG_INCLUDE_GATEWAYS,
                None,
                Some(adapters_ptr),
                &mut size,
            );

            if ret != 0 {
                return Err(Error::from_win32());
            }

            let mut current = adapters_ptr;
            let mut adapter_vec: Vec<Adapter> = Vec::new();

            while !current.is_null() {
                let adapter = &*current;

                let friendly_name = if !adapter.FriendlyName.is_null() {
                    adapter.FriendlyName.to_string().unwrap_or_default()
                } else {
                    String::new()
                };

                let description = if !adapter.Description.is_null() {
                    adapter.Description.to_string().unwrap_or_default()
                } else {
                    String::new()
                };

                let ipv4_addrs = Self::get_ipv4_addresses(adapter);
                let gateway = Self::get_default_gateway(adapter);

                adapter_vec.push(Adapter {
                    friendly_name,
                    description,
                    ipv4_addresses: ipv4_addrs,
                    gateways: gateway,
                });

                current = adapter.Next;
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

                if !sockaddr.is_null() && (*sockaddr).sa_family == AF_INET {
                    let ipv4 = *(sockaddr as *const SOCKADDR_IN);
                    let octets = ipv4.sin_addr.S_un.S_un_b;

                    ipv4_addrs.push(format!(
                        "{}.{}.{}.{}",
                        octets.s_b1, octets.s_b2, octets.s_b3, octets.s_b4
                    ));
                }

                current_unicast = unicast.Next;
            }

            if ipv4_addrs.is_empty() {
                None
            } else {
                Some(ipv4_addrs)
            }
        }
    }

    fn get_default_gateway(adapter: &IP_ADAPTER_ADDRESSES_LH) -> Option<String> {
        unsafe {
            let mut current_gw = adapter.FirstGatewayAddress;

            while !current_gw.is_null() {
                let gw = &*current_gw;
                let sockaddr = gw.Address.lpSockaddr;

                if !sockaddr.is_null() && (*sockaddr).sa_family == AF_INET {
                    let ipv4 = *(sockaddr as *const SOCKADDR_IN);
                    let octets = ipv4.sin_addr.S_un.S_un_b;

                    return Some(format!(
                        "{}.{}.{}.{}",
                        octets.s_b1, octets.s_b2, octets.s_b3, octets.s_b4
                    ));
                }

                current_gw = gw.Next;
            }

            None
        }
    }
}
