use serde::Serialize;
use windows::Win32::Foundation::*;
use windows::Win32::Security::*;
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::System::WindowsProgramming::GetUserNameW;
use windows::core::PWSTR;

#[derive(Serialize)]
pub struct UserInfo {
    pub username: Option<String>,
    pub groups: Option<Vec<String>>,
    pub privileges: Option<Vec<String>>,
}

impl UserInfo {
    pub fn collect() -> windows::core::Result<Self> {
        Ok(UserInfo {
            username: Self::get_username()?,
            groups: Self::get_user_groups()?,
            privileges: Self::get_user_privileges()?,
        })
    }

    fn get_username() -> windows::core::Result<Option<String>> {
        let mut buffer: [u16; 256] = [0; 256];
        let mut size = buffer.len() as u32;

        unsafe {
            GetUserNameW(PWSTR(buffer.as_mut_ptr()), &mut size)?;

            // size includes null terminator
            let len = (size - 1) as usize;
            Ok(Some(String::from_utf16_lossy(&buffer[..len]).to_owned()))
        }
    }

    fn get_user_groups() -> windows::core::Result<Option<Vec<String>>> {
        unsafe {
            let mut token_handle: HANDLE = HANDLE::default();
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle)?;

            let mut size = 0u32;
            // First call to get required size
            GetTokenInformation(token_handle, TokenGroups, None, 0, &mut size).ok();

            let mut buffer = vec![0u8; size as usize];

            GetTokenInformation(
                token_handle,
                TokenGroups,
                Some(buffer.as_mut_ptr() as *mut _),
                size,
                &mut size,
            )?;

            let token_groups: *const TOKEN_GROUPS = buffer.as_ptr() as *const TOKEN_GROUPS;
            let group_count = (*token_groups).GroupCount as usize;
            let groups_ptr = &(*token_groups).Groups as *const _ as *const SID_AND_ATTRIBUTES;

            let mut groups = Vec::with_capacity(group_count);

            for i in 0..group_count {
                let sid_and_attr = *groups_ptr.add(i);
                let sid = sid_and_attr.Sid;

                // Lookup name
                let mut name_buf: [u16; 256] = [0; 256];
                let mut name_len = name_buf.len() as u32;

                let mut domain_buf: [u16; 256] = [0; 256];
                let mut domain_len = domain_buf.len() as u32;

                let mut pe_use = SID_NAME_USE(0);

                LookupAccountSidW(
                    None,
                    sid,
                    PWSTR(name_buf.as_mut_ptr()),
                    &mut name_len,
                    PWSTR(domain_buf.as_mut_ptr()),
                    &mut domain_len,
                    &mut pe_use,
                )?;

                let name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
                let domain = String::from_utf16_lossy(&domain_buf[..domain_len as usize]);

                if domain.is_empty() {
                    groups.push(name.to_owned());
                } else {
                    groups.push(format!("{}\\{}", domain, name));
                }
            }

            Ok(Some(groups))
        }
    }

    fn get_user_privileges() -> windows::core::Result<Option<Vec<String>>> {
        unsafe {
            let mut token_handle: HANDLE = HANDLE::default();
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle)?;

            let mut size = 0u32;
            GetTokenInformation(token_handle, TokenPrivileges, None, 0, &mut size).ok();

            let mut buffer = vec![0u8; size as usize];

            GetTokenInformation(
                token_handle,
                TokenPrivileges,
                Some(buffer.as_mut_ptr() as *mut _),
                size,
                &mut size,
            )?;

            let token_privileges: *const TOKEN_PRIVILEGES =
                buffer.as_ptr() as *const TOKEN_PRIVILEGES;
            let count = (*token_privileges).PrivilegeCount as usize;

            let privs_ptr =
                &(*token_privileges).Privileges as *const _ as *const LUID_AND_ATTRIBUTES;

            let mut privileges = Vec::with_capacity(count);

            for i in 0..count {
                let laa = *privs_ptr.add(i);

                let mut name_buf: [u16; 256] = [0; 256];
                let mut name_len = name_buf.len() as u32;

                LookupPrivilegeNameW(None, &laa.Luid, PWSTR(name_buf.as_mut_ptr()), &mut name_len)?;

                let name = String::from_utf16_lossy(&name_buf[..name_len as usize]);

                let enabled =
                    laa.Attributes & SE_PRIVILEGE_ENABLED != TOKEN_PRIVILEGES_ATTRIBUTES(0);

                privileges.push(format!(
                    "{}{}",
                    name,
                    if enabled { " (Enabled)" } else { "" }
                ));
            }

            Ok(Some(privileges))
        }
    }
}
