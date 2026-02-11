use std::collections::HashMap;
use std::process::Command;
use sysinfo::{Disks, Networks, System};

pub fn get_cpu_usage(sys: &System) -> f64 {
    sys.global_cpu_usage() as f64
}

pub fn get_memory(sys: &System) -> (u64, u64) {
    (sys.used_memory(), sys.total_memory())
}

pub fn get_disk_io(_disks: &Disks) -> (u64, u64) {
    // Use /proc/diskstats on Linux or ioreg on macOS
    if cfg!(target_os = "linux") {
        if let Ok(content) = std::fs::read_to_string("/proc/diskstats") {
            let mut read_bytes = 0u64;
            let mut write_bytes = 0u64;
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 14 {
                    // Only count real disks (sd*, nvme*, vd*), not partitions
                    let name = parts[2];
                    if name.starts_with("sd") || name.starts_with("nvme") || name.starts_with("vd") {
                        // Skip partitions (e.g., sda1, nvme0n1p1)
                        let is_partition = name.chars().last().map(|c| c.is_numeric()).unwrap_or(false)
                            && name.contains(|c: char| c.is_numeric());
                        if !is_partition || name.starts_with("nvme") {
                            // Fields: reads completed (3), sectors read (5), writes completed (7), sectors written (9)
                            if let (Ok(read_sectors), Ok(write_sectors)) =
                                (parts[5].parse::<u64>(), parts[9].parse::<u64>())
                            {
                                read_bytes += read_sectors * 512;
                                write_bytes += write_sectors * 512;
                            }
                        }
                    }
                }
            }
            return (read_bytes, write_bytes);
        }
    } else if cfg!(target_os = "macos") {
        // On macOS, use ioreg to get disk I/O statistics
        // This gives us proper separate read/write byte counts
        if let Ok(output) = Command::new("ioreg")
            .args(["-c", "IOBlockStorageDriver", "-r", "-d", "1"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut total_read = 0u64;
            let mut total_write = 0u64;

            for line in stdout.lines() {
                if line.contains("Statistics") {
                    // Parse "Bytes (Read)"=NNNN
                    if let Some(read_start) = line.find("\"Bytes (Read)\"=") {
                        let start = read_start + 15;
                        let rest = &line[start..];
                        if let Some(end) = rest.find(|c: char| !c.is_numeric()) {
                            if let Ok(bytes) = rest[..end].parse::<u64>() {
                                total_read += bytes;
                            }
                        }
                    }
                    // Parse "Bytes (Write)"=NNNN
                    if let Some(write_start) = line.find("\"Bytes (Write)\"=") {
                        let start = write_start + 16;
                        let rest = &line[start..];
                        if let Some(end) = rest.find(|c: char| !c.is_numeric()) {
                            if let Ok(bytes) = rest[..end].parse::<u64>() {
                                total_write += bytes;
                            }
                        }
                    }
                }
            }
            return (total_read, total_write);
        }
    }

    (0, 0)
}

pub fn get_network_io(networks: &Networks) -> (u64, u64) {
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;

    for (_name, data) in networks.list() {
        total_rx += data.total_received();
        total_tx += data.total_transmitted();
    }

    (total_rx, total_tx)
}

pub fn get_open_ports(sys: &System) -> Vec<(u16, String, u32)> {
    let mut ports: Vec<(u16, String, u32)> = Vec::new();

    // Build PID to process name map
    let mut pid_to_name: HashMap<u32, String> = HashMap::new();
    for (pid, process) in sys.processes() {
        pid_to_name.insert(pid.as_u32(), process.name().to_string_lossy().to_string());
    }

    if cfg!(target_os = "macos") {
        ports = get_ports_macos(&pid_to_name);
    } else if cfg!(target_os = "linux") {
        ports = get_ports_linux(&pid_to_name);
    }

    // Sort by port number
    ports.sort_by_key(|(port, _, _)| *port);
    ports
}

pub fn kill_process(pid: u32) -> bool {
    use std::process::Command;

    let result = if cfg!(target_os = "windows") {
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output()
    } else {
        Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output()
    };

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn get_ports_macos(pid_to_name: &HashMap<u32, String>) -> Vec<(u16, String, u32)> {
    let mut ports = Vec::new();

    // Use lsof to get listening ports
    if let Ok(output) = Command::new("lsof")
        .args(["-i", "-P", "-n"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if !line.contains("LISTEN") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 9 {
                continue;
            }

            // Format: COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
            let command = parts[0];
            let pid_str = parts[1];
            let name_field = parts[8]; // e.g., "*:8080" or "127.0.0.1:3000"

            // Extract port from name field
            if let Some(port_str) = name_field.rsplit(':').next() {
                if let Ok(port) = port_str.parse::<u16>() {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        // Get process name from pid_to_name or use command
                        let process_name = pid_to_name
                            .get(&pid)
                            .cloned()
                            .unwrap_or_else(|| command.to_string());

                        // Avoid duplicates
                        if !ports.iter().any(|(p, _, _)| *p == port) {
                            ports.push((port, process_name, pid));
                        }
                    }
                }
            }
        }
    }

    ports
}

fn get_ports_linux(pid_to_name: &HashMap<u32, String>) -> Vec<(u16, String, u32)> {
    let mut ports = Vec::new();

    // Use ss to get listening ports
    if let Ok(output) = Command::new("ss")
        .args(["-tlnp"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }

            // Format: State Recv-Q Send-Q Local Address:Port Peer Address:Port Process
            let local_addr = parts[3];

            // Extract port
            if let Some(port_str) = local_addr.rsplit(':').next() {
                if let Ok(port) = port_str.parse::<u16>() {
                    // Try to extract PID from process column
                    let mut pid: u32 = 0;
                    let process_name = if parts.len() > 5 {
                        let process_info = parts[5..].join(" ");
                        // Format: users:(("process",pid=1234,fd=5))
                        if let Some(start) = process_info.find("pid=") {
                            let pid_start = start + 4;
                            let pid_end = process_info[pid_start..]
                                .find(|c: char| !c.is_numeric())
                                .map(|i| pid_start + i)
                                .unwrap_or(process_info.len());
                            if let Ok(parsed_pid) = process_info[pid_start..pid_end].parse::<u32>() {
                                pid = parsed_pid;
                                pid_to_name
                                    .get(&pid)
                                    .cloned()
                                    .unwrap_or_else(|| "unknown".to_string())
                            } else {
                                "unknown".to_string()
                            }
                        } else {
                            "unknown".to_string()
                        }
                    } else {
                        "unknown".to_string()
                    };

                    if pid > 0 && !ports.iter().any(|(p, _, _)| *p == port) {
                        ports.push((port, process_name, pid));
                    }
                }
            }
        }
    }

    ports
}
