use std::process::Command;

fn main() {
	println!("Testing VPN detection");

	// Get OS information
	let os = std::env::consts::OS;
	println!("OS: {}", os);

	// Test macOS detection if on macOS
	if os == "macos" {
		println!("\nTesting macOS VPN detection:");

		// Check for common VPN interfaces using networksetup
		let output = Command::new("networksetup").args(["-listallnetworkservices"]).output();

		if let Ok(output) = output {
			let services = String::from_utf8_lossy(&output.stdout);
			println!("Network services:");
			println!("{}", services);

			// Check for common VPN service names
			if services.contains("VPN")
				|| services.contains("Cisco")
				|| services.contains("OpenVPN")
				|| services.contains("Pulse")
				|| services.contains("Tunnelblick")
				|| services.contains("WireGuard")
			{
				println!("VPN service detected");
			} else {
				println!("No VPN service detected in network services");
			}
		} else {
			println!("Failed to run networksetup command");
		}

		// Alternative detection using routing table
		let output = Command::new("netstat").args(["-nr"]).output();

		if let Ok(output) = output {
			let routes = String::from_utf8_lossy(&output.stdout);
			println!("\nRouting table:");
			println!("{}", routes);

			// Look for common VPN-related entries in routing table
			if routes.contains("tun") || routes.contains("utun") || routes.contains("ppp") {
				println!("VPN interface detected in routing table");
			} else {
				println!("No VPN interface detected in routing table");
			}
		} else {
			println!("Failed to run netstat command");
		}
	}

	// Test Windows detection if on Windows
	if os == "windows" {
		println!("\nTesting Windows VPN detection:");

		// Check network interfaces using ipconfig
		let output = Command::new("ipconfig").args(["/all"]).output();

		if let Ok(output) = output {
			let interfaces = String::from_utf8_lossy(&output.stdout);
			println!("Network interfaces:");
			println!("{}", interfaces);

			// Check for common VPN adapter descriptions
			if interfaces.contains("VPN")
				|| interfaces.contains("Virtual")
				|| interfaces.contains("Cisco")
				|| interfaces.contains("OpenVPN")
				|| interfaces.contains("Pulse")
				|| interfaces.contains("WireGuard")
			{
				println!("VPN adapter detected");
			} else {
				println!("No VPN adapter detected");
			}
		} else {
			println!("Failed to run ipconfig command");
		}
	}

	// Test Linux detection if on Linux
	if os == "linux" {
		println!("\nTesting Linux VPN detection:");

		// Check for common VPN interfaces in /proc/net/dev
		if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
			println!("Network interfaces in /proc/net/dev:");
			println!("{}", content);

			if content.contains("tun") || content.contains("tap") || content.contains("ppp") {
				println!("VPN interface detected in /proc/net/dev");
			} else {
				println!("No VPN interface detected in /proc/net/dev");
			}
		} else {
			println!("Failed to read /proc/net/dev");
		}

		// Check running processes for VPN services
		let output = Command::new("ps").args(["-A"]).output();

		if let Ok(output) = output {
			let processes = String::from_utf8_lossy(&output.stdout);
			println!("\nRunning processes:");
			println!("{}", processes);

			// Check for common VPN process names
			if processes.contains("openvpn")
				|| processes.contains("vpnc")
				|| processes.contains("openconnect")
				|| processes.contains("wireguard")
			{
				println!("VPN process detected");
			} else {
				println!("No VPN process detected");
			}
		} else {
			println!("Failed to run ps command");
		}
	}

	println!("\nVPN detection test complete");
}
