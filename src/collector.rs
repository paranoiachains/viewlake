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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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
    fn collect_system_fingerprint() {
        let sys_fingerprint = SystemFingerprint::collect()?;
    }
}
