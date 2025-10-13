//! Debug utilities for Docker container troubleshooting

/// Debug helper function to collect Docker container information for troubleshooting
///
/// This function runs various Docker commands to help diagnose issues when SSH
/// connectivity tests fail in CI environments. It prints container status, logs,
/// and other useful debugging information.
///
/// # Arguments
///
/// * `container_port` - The host port that the SSH container is mapped to
///
/// # Usage
///
/// This function is typically called when SSH connectivity tests fail to help
/// diagnose what's happening with the Docker containers in CI environments.
///
/// ```rust
/// use torrust_tracker_deployer_lib::testing::integration::ssh_server::print_docker_debug_info;
///
/// // In a test when SSH connectivity fails:
/// print_docker_debug_info(2222);
/// ```
pub fn print_docker_debug_info(container_port: u16) {
    println!("\n=== Docker Debug Information ===");

    // Check if Docker is running and list all containers
    match std::process::Command::new("docker")
        .args(["ps", "-a"])
        .output()
    {
        Ok(output) => {
            println!("Docker containers (docker ps -a):");
            println!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.stderr.is_empty() {
                println!(
                    "Docker ps stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            println!("Failed to run 'docker ps -a': {e}");
        }
    }

    // Check Docker images
    match std::process::Command::new("docker")
        .args(["images", "torrust-ssh-server"])
        .output()
    {
        Ok(output) => {
            println!("\nDocker images for torrust-ssh-server:");
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        Err(e) => {
            println!("Failed to run 'docker images': {e}");
        }
    }

    // Try to find containers with the SSH image
    match std::process::Command::new("docker")
        .args(["ps", "-a", "--filter", "ancestor=torrust-ssh-server:latest"])
        .output()
    {
        Ok(output) => {
            println!("\nContainers using torrust-ssh-server:latest:");
            println!("{}", String::from_utf8_lossy(&output.stdout));

            // Get container logs if there are any containers
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout_str.lines().nth(1) {
                // Skip header, get first container
                if let Some(container_id) = line.split_whitespace().next() {
                    println!("\nContainer logs for {container_id}:");
                    match std::process::Command::new("docker")
                        .args(["logs", "--tail", "20", container_id])
                        .output()
                    {
                        Ok(log_output) => {
                            println!("{}", String::from_utf8_lossy(&log_output.stdout));
                            if !log_output.stderr.is_empty() {
                                println!(
                                    "Container stderr: {}",
                                    String::from_utf8_lossy(&log_output.stderr)
                                );
                            }
                        }
                        Err(e) => {
                            println!("Failed to get container logs: {e}");
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to filter Docker containers: {e}");
        }
    }

    // Check if the specific port is being used
    println!("\nPort information:");
    println!("Expected SSH port mapping: host -> container:22");
    match std::process::Command::new("netstat")
        .args(["-tlnp"])
        .output()
    {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains(&container_port.to_string()) {
                    println!("Port {container_port} usage: {line}");
                }
            }
        }
        Err(_) => {
            // Fallback to ss command if netstat is not available
            match std::process::Command::new("ss").args(["-tlnp"]).output() {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    for line in output_str.lines() {
                        if line.contains(&container_port.to_string()) {
                            println!("Port {container_port} usage (ss): {line}");
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to check port usage (netstat/ss not available): {e}");
                }
            }
        }
    }

    println!("=== End Docker Debug Information ===\n");
}
