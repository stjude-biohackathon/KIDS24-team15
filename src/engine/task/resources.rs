//! Task resource specifications.

mod builder;

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
