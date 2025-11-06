use windows::Win32::Foundation::*;
use windows::Win32::Security::*;
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::Result;
use windows::{Win32::System::WindowsProgramming::GetUserNameW, core::PWSTR};

pub struct UserInfo {
    pub username: Option<String>,
    pub groups: Option<Vec<String>>,
    pub privileges: Option<Vec<String>>,
}

impl UserInfo {
    pub fn collect() -> Result<Self> {
        Ok(UserInfo {
            username: Self::get_username()?,
            groups: Self::get_user_groups()?,
            privileges: Self::get_user_privileges()?,
        })
    }

    fn get_username() -> Result<Option<String>> {
        let mut buffer: [u16; 256] = [0; 256];
        #[allow(unused_mut)]
        let mut size = buffer.len() as u32;
        unsafe {
            GetUserNameW(PWSTR::from_raw(buffer.as_mut_ptr()), &mut size)?;
            Ok(Some(String::from_utf16_lossy(
                &buffer[..(size - 1) as usize],
            )))
        }
    }

    fn get_user_groups() -> Result<Option<Vec<String>>> {
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

            Ok(Some(groups))
        }
    }

    fn get_user_privileges() -> Result<Option<Vec<String>>> {
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
                let laa = *privileges_ptr.add(i);

                let mut name_buf = [0u16; 256];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_user_info() -> Result<()> {
        let info = UserInfo::collect()?;
        println!("Username: {}", info.username.as_ref().unwrap());
        assert!(info.username.is_some(), "Username should not be empty");

        assert!(info.groups.is_some(), "Groups should not be None");
        assert!(
            !info.groups.as_ref().unwrap().is_empty(),
            "Groups should not be empty"
        );
        println!("Groups: {:?}", info.groups.unwrap());

        assert!(info.privileges.is_some(), "Privileges should not be None");
        assert!(
            !info.privileges.as_ref().unwrap().is_empty(),
            "Privileges should not be empty"
        );
        println!("Privileges: {:?}", info.privileges.unwrap());

        Ok(())
    }
}
