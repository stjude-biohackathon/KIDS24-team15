//! Task resource specifications.

mod builder;

use std::collections::HashMap;

use bollard::secret::HostConfig;
pub use builder::Builder;

use nonempty::NonEmpty;

/// A set of requested resources.
#[derive(Clone, Debug)]
pub struct Resources {
    /// The number of CPU cores requested.
    cpu_cores: Option<u64>,

    /// Whether or not the task may use preemptible resources.
    preemptible: Option<bool>,

    /// The requested random access memory size in gigabytes.
    ram_gb: Option<f64>,

    /// The requested disk size in gigabytes.
    disk_gb: Option<f64>,

    /// The associated compute zones.
    zones: Option<NonEmpty<String>>,
}

impl Resources {
    /// A number of CPU cores.
    pub fn cpu_cores(&self) -> Option<u64> {
        self.cpu_cores
    }

    /// Whether the instance should be preemptible.
    pub fn preemptible(&self) -> Option<bool> {
        self.preemptible
    }

    /// The amount of RAM in gigabytes.
    pub fn ram_gb(&self) -> Option<f64> {
        self.ram_gb
    }

    /// The amount of disk space in gigabytes.
    pub fn disk_gb(&self) -> Option<f64> {
        self.disk_gb
    }

    /// The set of requested zones.
    pub fn zones(&self) -> Option<&NonEmpty<String>> {
        self.zones.as_ref()
    }
}

impl From<&Resources> for HostConfig {
    fn from(resources: &Resources) -> Self {
        let mut host_config = HostConfig::default();
        if let Some(ram_gb) = resources.ram_gb() {
            host_config.memory = Some((ram_gb * 1024. * 1024. * 1024.) as i64);
        }

        if let Some(cpu_cores) = resources.cpu_cores() {
            host_config.cpu_count = Some(cpu_cores as i64);
        }

        if let Some(disk_gb) = resources.disk_gb() {
            let mut storage_opt: HashMap<String, String> = HashMap::new();
            storage_opt.insert("size".to_string(), disk_gb.to_string());
            host_config.storage_opt = Some(storage_opt);
        }

        host_config
    }
}
