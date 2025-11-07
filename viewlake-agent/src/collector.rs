#![allow(dead_code)]
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
#[derive(serde::Serialize)]
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
        let user = UserInfo::collect()?;

        Ok(Self {
            network,
            processes,
            system,
            user,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_system_fingerprint_success() {
        let result = SystemFingerprint::collect();
        assert!(
            result.is_ok(),
            "SystemFingerprint::collect() returned an error"
        );

        let fingerprint = result.unwrap();

        assert!(
            fingerprint.network.adapters.is_some(),
            "fingerprint.network.adapters_info shouldn't be Err"
        );

        assert!(
            !fingerprint.processes.list.is_empty(),
            "Expected at least one running process"
        );

        assert!(
            !fingerprint.system.version.is_empty(),
            "OS name should not be empty: {}",
            fingerprint.system.version
        );
        assert!(
            !fingerprint.system.arch.is_empty(),
            "ARCH shouldn't be empty"
        );

        assert!(
            fingerprint.user.username.is_some(),
            "Username shouldn't be None"
        );
    }
}
