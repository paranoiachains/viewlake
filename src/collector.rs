use windows::core::Error;

use crate::collector::{
    network::NetworkInfo, processes::ProcessList, system::SystemInfo, user::UserInfo,
};

mod network;
mod processes;
mod system;
mod user;

pub struct SystemFingerprint {
    pub network: NetworkInfo,
    pub processes: ProcessList,
    pub system: SystemInfo,
    pub user: UserInfo,
}

impl SystemFingerprint {
    pub fn collect() -> Result<Self, Error> {
        let network = NetworkInfo::collect()?;
        let processes = ProcessList::collect()?;
        let system = SystemInfo::collect()?;
        let user = UserInfo::collect()?;

        Ok(Self {
            network,
            processes,
            system,
            user,
        })
    }
}

impl Display for SystemFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<--Network-->\n
            {}",
            self.network
        )?;
        write!(
            f,
            "<--Processes-->\n
            {}",
            self.processes
        )?;
        write!(
            f,
            "<--System-->\n
            {}",
            self.system
        )?;
        write!(
            f,
            "<--User-->\n
            {}",
            self.user
        )?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_system_fingerprint() {
        let sys_fingerprint = SystemFingerprint::collect()?;
    }
}
