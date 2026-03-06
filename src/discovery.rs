use std::fmt;
use sysinfo::Disks;

#[derive(Debug, Clone)]
pub struct BlockDevice {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub mount_point: Option<String>,
}

impl fmt::Display for BlockDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size_gb = self.size as f64 / 1_073_741_824.0;
        let mount_info = match &self.mount_point {
            Some(mp) => format!(" (Mounted on {})", mp),
            None => "".to_string(),
        };
        write!(
            f,
            "{} - {:.2} GB - {}{}",
            self.path, size_gb, self.name, mount_info
        )
    }
}

pub fn list_block_devices() -> Vec<BlockDevice> {
    let disks = Disks::new_with_refreshed_list();
    let mut devices = Vec::new();

    for disk in &disks {
        devices.push(BlockDevice {
            name: disk.name().to_string_lossy().to_string(),
            path: disk.mount_point().to_string_lossy().to_string(), // Initial attempt
            size: disk.total_space(),
            mount_point: Some(disk.mount_point().to_string_lossy().to_string()),
        });
    }

    // Since sysinfo might only list partitions, on Linux we can fallback to /sys/class/block
    // or manual detection if needed for raw physical drives.
    // For now, sysinfo provides a good base.

    devices
}
