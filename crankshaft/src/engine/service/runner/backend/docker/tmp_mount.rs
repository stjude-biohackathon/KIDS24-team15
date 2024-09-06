//! Create local temporary folders to mount in docker containers

use std::str::FromStr;

use bollard::models::Mount;
use bollard::models::MountTypeEnum;
use tempfile::TempDir;

/// Mount a local TempDir to a mount shared amongst the executors in a tast
pub struct TmpMount {
    /// local temp directory mounted as a volume
    local_path: TempDir,

    /// path used to mount in the container
    container_path: String,
}

impl FromStr for TmpMount {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TmpMount {
            local_path: TempDir::new()?,
            container_path: s.to_string(),
        })
    }
}

impl From<&TmpMount> for Mount {
    fn from(val: &TmpMount) -> Self {
        Mount {
            target: Some(val.container_path.clone()),
            source: Some(val.local_path.path().to_str().unwrap().to_string()),
            typ: Some(MountTypeEnum::BIND),
            ..Default::default()
        }
    }
}
