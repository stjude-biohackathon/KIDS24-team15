//! Generic backend implementation
//!
//! Note: Generic backend isn't actually constructed currently. Once we have a `Backend` trait, I'd like to add a `to_backend` method to `BackendConfig`
//! that returns something like a `Box<dyn Backend>.` For now since we don't have this yet, I just have the submit method directly in `BackendConfig`

/// A generic backend
pub struct GenericBackend {
    /// Default cpu count
    pub default_cpu: Option<u32>,
    /// Default ram amount in mb
    pub default_ram_mb: Option<u32>,
    /// command to run on submit
    pub submit: String,
}
