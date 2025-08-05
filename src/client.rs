use windows::core::PCWSTR;

use crate::winhttp::{WinHttpConnection, WinHttpSession};
use windows::core::Error;

pub struct Connection {
    pub session: WinHttpSession,
    pub connection: WinHttpConnection,
}

pub struct Client {
    pub session: WinHttpSession,
    pub connection: Option<WinHttpConnection>,
}

impl Client {
    pub fn new(agent: &str, hostname: Option<&str>) -> Result<Self, Error> {
        let agent_wide = PCWSTR::from_raw(agent.as_ptr() as *const u16);

        let session = match WinHttpSession::new(agent_wide) {
            Ok(session) => session,
            Err(e) => return Err(e),
        };
        let connection = if let Some(host) = hostname {
            let host_wide = PCWSTR::from_raw(host.as_ptr() as *const u16);
            Some(WinHttpConnection::new(&session, host_wide)?)
        } else {
            None
        };

        Ok(Client {
            session,
            connection,
        })
    }
}
