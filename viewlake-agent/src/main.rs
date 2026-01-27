#![no_main]

use log::{error, info};
use viewlake_agent::run;
use windows::core::HSTRING;

#[unsafe(no_mangle)]
unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> u32 {
    #[cfg(feature = "logging")]
    viewlake_agent::init_logging();

    info!("started logger");

    let mut args = std::env::args_os();
    args.next(); // skip exe name

    let hostname_os = match args.next() {
        Some(h) => h,
        None => {
            error!("hostname is required");
            return 1;
        }
    };

    let port_os = match args.next() {
        Some(p) => p,
        None => {
            error!("port is required");
            return 1;
        }
    };

    let hostname = HSTRING::from(hostname_os.as_os_str());

    let port: u16 = match port_os.to_string_lossy().parse() {
        Ok(p) => p,
        Err(_) => {
            error!("port must be a number");
            return 1;
        }
    };

    if let Err(e) = run(&hostname, port) {
        error!("got error: {}", e);
    }

    0
}
