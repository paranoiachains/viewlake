use windows::Win32::Foundation::*;
use windows::Win32::Security::{
    GetTokenInformation, LookupAccountSidW, SID_AND_ATTRIBUTES, SID_NAME_USE, TOKEN_GROUPS,
    TOKEN_QUERY, TokenGroups,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::{Win32::System::WindowsProgramming::GetUserNameW, core::PWSTR};

#[derive(Debug)]
pub struct UserInfo {
    pub username: Option<String>,
    pub groups: Vec<String>,
}

impl UserInfo {
    pub fn collect() -> Self {
        UserInfo {
            username: Self::get_username().ok(),
            groups: Self::get_user_groups().expect("Couldn't retrieve groups"),
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
                groups.push(format!("{}\\{}", group, domain));
            }

            Ok(groups)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_user_info() {
        let uinfo = UserInfo::collect();
        println!("username: {:?}", uinfo.username);
        println!("token: {:?}", uinfo.groups);
    }
}
