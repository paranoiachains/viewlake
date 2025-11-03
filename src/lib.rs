pub mod collector;
pub mod comms;

use collector::SystemFingerprint;
use windows::core::Result;

pub fn collect() -> Result<SystemFingerprint> {
    SystemFingerprint::collect()
}
