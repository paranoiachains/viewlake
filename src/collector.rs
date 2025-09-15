use windows::core::Result;

use crate::collector::{
    network::NetworkInfo, processes::ProcessList, system::SystemInfo, user::UserInfo,
};

mod network;
mod processes;
mod system;
mod user;

/// Represents a complete fingerprint of the system.
/// Collects network, processes, system info, and user info.
pub struct SystemFingerprint {
    pub network: NetworkInfo,
    pub processes: ProcessList,
    pub system: SystemInfo,
    pub user: UserInfo,
}

impl SystemFingerprint {
    pub fn collect() -> Result<Self> {
        let network = NetworkInfo::collect()?;
        let processes = ProcessList::collect()?;
        let system = SystemInfo::collect()?;
        let user = UserInfo::collect();

        Ok(Self {
            network,
            processes,
            system,
            user,
        })
    }
}

impl std::fmt::Display for SystemFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Network:\n{}\n\
             Processes:\n{}\n\
             System:\n{}\n\
             User:\n{}",
            self.network, self.processes, self.system, self.user
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_system_fingerprint() -> Result<()> {
        let sys_fingerprint = SystemFingerprint::collect()?;
        assert!(!sys_fingerprint.network.adapters_info.is_ok());
        assert!(!sys_fingerprint.processes.list.is_empty());
        Ok(())
    }
}
