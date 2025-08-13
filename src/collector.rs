// This logic implies only logging/collecting, not pattern matching
pub mod network;
pub mod system;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_system_info() {
        let os_info = system::SystemInfo::collect().expect("Failed to get OS info");

        println!("Arch: {}", os_info.arch);
        println!("OS Version: {}", os_info.version);
        println!("Product Type: {}", os_info.product_type);

        assert!(
            os_info.arch == "AMD64" || os_info.arch == "ARM64" || os_info.arch == "Unknown",
            "Unexpected architecture: {}",
            os_info.arch
        );

        assert!(
            os_info.version.chars().next().unwrap().is_ascii_digit(),
            "OS version is not a valid number: {}",
            os_info.version
        );

        assert!(
            os_info.product_type == "Workstation"
                || os_info.product_type == "Server"
                || os_info.product_type == "Domain Controller"
                || os_info.product_type == "Unknown",
            "Unexpected product type: {}",
            os_info.product_type
        );
    }

    #[test]
    fn get_network_info() {
        let nwinfo = network::NetworkInfo::collect().expect("Failed to collect network info");
        println!("hostname: {}", nwinfo.hostname);
        println!("domain_or_workgroup: {}", nwinfo.domain_or_workgroup);
        println!("status: {}", nwinfo.status);
        println!("adapters: {:?}", nwinfo.adapters_info);
    }
}
