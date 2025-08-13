use windows::Win32::Foundation::*;
use windows::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TokenUser};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::{Win32::System::WindowsProgramming::GetUserNameW, core::PWSTR};

#[derive(Debug)]
pub struct UserInfo {
    pub username: Option<String>,
    pub token: Option<String>,
}

impl UserInfo {
    pub fn collect() -> Self {
        UserInfo {
            username: Self::get_username().ok(),
            token: Self::get_security_token().ok(),
        }
    }

    fn get_username() -> Result<String, windows::core::Error> {
        let mut buffer: [u16; 256] = [0; 256];
        #[allow(unused_mut)]
        let mut size = buffer.len() as u32;
        unsafe {
            GetUserNameW(PWSTR::from_raw(buffer.as_mut_ptr()), size as *mut u32)?;
            Ok(String::from_utf16_lossy(&buffer[..(size - 1) as usize]))
        }
    }

    fn get_security_token() -> Result<String, windows::core::Error> {
        unsafe {
            let mut token_handle: HANDLE = HANDLE::default();
            let handle = GetCurrentProcess();

            OpenProcessToken(handle, TOKEN_QUERY, &mut token_handle)?;

            let mut return_length: u32 = 0;

            GetTokenInformation(token_handle, TokenUser, None, 0, &mut return_length)?;
            let mut buffer: Vec<u8> = vec![0; return_length as usize];

            GetTokenInformation(
                token_handle,
                TokenUser,
                Some(buffer.as_mut_ptr() as *mut _),
                return_length,
                &mut return_length,
            )?;
            Ok(String::from_utf8(buffer).expect("Couldn't convert username to string"))
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
        println!("token: {:?}", uinfo.token);
    }
}
