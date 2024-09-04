//! Task resource specifications.

use nonempty::NonEmpty;
use tracing::warn;

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

/// A builder for [`Resources`].
#[derive(Debug, Default)]
pub struct ResourcesBuilder {
    cpu_cores: Option<u64>,
    preemptible: Option<bool>,
    ram_gb: Option<f64>,
    disk_gb: Option<f64>,
    zones: Option<NonEmpty<String>>,
}

impl Resources {
    /// Gets a new [`ResourcesBuilder`].
    pub fn builder() -> ResourcesBuilder {
        ResourcesBuilder::default()
    }
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

impl ResourcesBuilder {
    /// Sets the number of CPU cores.
    pub fn cpu_cores(mut self, cpu_cores: u64) -> Self {
        self.cpu_cores = Some(cpu_cores);
        self
    }

    /// Sets whether the instance should be preemptible.
    pub fn preemptible(mut self, preemptible: bool) -> Self {
        self.preemptible = Some(preemptible);
        self
    }

    /// Sets the amount of RAM in gigabytes.
    pub fn ram_gb(mut self, ram_gb: f64) -> Self {
        self.ram_gb = Some(ram_gb);
        self
    }

    /// Sets the amount of disk space in gigabytes.
    pub fn disk_gb(self, _disk_gb: f64) -> Self {
        warn!("Setting Disk Space does not work on containers unless XFS is used on the host machine");
        self
    }

    /// Sets the set of requested zones.
    pub fn zones(mut self, zones: NonEmpty<String>) -> Self {
        self.zones = Some(zones);
        self
    }

    /// Builds the [`Resources`].
    pub fn build(self) -> Resources {
        Resources {
            cpu_cores: self.cpu_cores,
            preemptible: self.preemptible,
            ram_gb: self.ram_gb,
            disk_gb: self.disk_gb,
            zones: self.zones,
        }
    }
}
