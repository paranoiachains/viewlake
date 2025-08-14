use windows::Win32::Foundation::*;
use windows::Win32::Security::*;
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::{Win32::System::WindowsProgramming::GetUserNameW, core::PWSTR};

#[derive(Debug)]
pub struct UserInfo {
    pub username: Option<String>,
    pub groups: Option<Vec<String>>,
    pub privileges: Option<Vec<String>>,
}

impl UserInfo {
    pub fn collect() -> Self {
        UserInfo {
            username: Self::get_username().ok(),
            groups: Self::get_user_groups().ok(),
            privileges: Self::get_user_privileges().ok(),
        }
    }

    fn get_username() -> Result<String, windows::core::Error> {
        let mut buffer: [u16; 256] = [0; 256];
        #[allow(unused_mut)]
        let mut size = buffer.len() as u32;
        unsafe {
            GetUserNameW(PWSTR::from_raw(buffer.as_mut_ptr()), &mut size)?;
            Ok(String::from_utf16_lossy(&buffer[..(size - 1) as usize]))
        }
    }

    fn get_user_groups() -> Result<Vec<String>, windows::core::Error> {
        unsafe {
            let mut token_handle: HANDLE = HANDLE::default();
            let handle = GetCurrentProcess();

            OpenProcessToken(handle, TOKEN_QUERY, &mut token_handle)?;

            let mut return_length: u32 = 1024;
            let mut buffer: Vec<u8> = vec![0; return_length as usize];

            GetTokenInformation(
                token_handle,
                TokenGroups,
                Some(buffer.as_mut_ptr() as *mut _),
                return_length,
                &mut return_length,
            )?;

            let token_groups: *const TOKEN_GROUPS = buffer.as_ptr() as *const TOKEN_GROUPS;
            let group_count = (*token_groups).GroupCount as usize;

            let groups_ptr = &(*token_groups).Groups as *const _ as *const SID_AND_ATTRIBUTES;

            let mut groups = Vec::new();

            for i in 0..group_count {
                let sid_and_attr = *groups_ptr.add(i);
                let sid = sid_and_attr.Sid;

                let mut name = [0u16; 256];
                let mut cch_name = name.len() as u32;
                let mut domain = [0u16; 256];
                let mut cch_domain = domain.len() as u32;
                let mut pe_use = SID_NAME_USE(0);

                LookupAccountSidW(
                    None,
                    sid,
                    PWSTR(name.as_mut_ptr()),
                    &mut cch_name,
                    PWSTR(domain.as_mut_ptr()),
                    &mut cch_domain,
                    &mut pe_use,
                )?;

                let group = String::from_utf16_lossy(&name[..cch_name as usize]);
                let domain = String::from_utf16_lossy(&domain[..cch_domain as usize]);

                if domain.is_empty() {
                    groups.push(group)
                } else {
                    groups.push(format!("{}\\{}", domain, group));
                }
            }

            Ok(groups)
        }
    }

    fn get_user_privileges() -> Result<Vec<String>, windows::core::Error> {
        unsafe {
            let mut token_handle: HANDLE = HANDLE::default();
            let handle = GetCurrentProcess();

            OpenProcessToken(handle, TOKEN_QUERY, &mut token_handle)?;

            let mut return_length: u32 = 1024;
            let mut buffer: Vec<u8> = vec![0; return_length as usize];

            GetTokenInformation(
                token_handle,
                TokenPrivileges,
                Some(buffer.as_mut_ptr() as *mut _),
                return_length,
                &mut return_length,
            )?;

            let token_privileges: *const TOKEN_PRIVILEGES =
                buffer.as_ptr() as *const TOKEN_PRIVILEGES;
            let privilege_count = (*token_privileges).PrivilegeCount as usize;

            let privileges_ptr =
                &(*token_privileges).Privileges as *const _ as *const LUID_AND_ATTRIBUTES;
            let mut privileges = Vec::new();

            for i in 0..privilege_count {
                let laa = unsafe { *privileges_ptr.add(i) };

                let mut name_buf = [0u16; 256];
                let mut name_len = name_buf.len() as u32;

                unsafe {
                    LookupPrivilegeNameW(
                        None,
                        &laa.Luid,
                        PWSTR(name_buf.as_mut_ptr()),
                        &mut name_len,
                    )?;
                }

                let name = String::from_utf16_lossy(&name_buf[..name_len as usize]);

                let enabled = laa.Attributes & SE_PRIVILEGE_ENABLED.0 != 0;

                privileges.push(format!(
                    "{}{}",
                    name,
                    if enabled { " (Enabled)" } else { "" }
                ));
            }

            Ok(privileges)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_user_info() {
        let info = UserInfo::collect();
        println!("{:?}", info);

        assert!(info.username.is_some(), "Username should not be None");
    }
}
