use serde::Serialize;

use crate::collector::{
    network::NetworkInfo, processes::ProcessList, system::SystemInfo, user::UserInfo,
};

mod network;
mod processes;
mod system;
mod user;

#[derive(Serialize)]
pub struct SystemFingerprint {
    network: NetworkInfo,
    processes: ProcessList,
    system: SystemInfo,
    user: UserInfo,
}

impl SystemFingerprint {
    pub fn collect() -> windows::core::Result<Self> {
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
