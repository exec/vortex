use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Use dirs crate for secure home directory detection
use dirs::home_dir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub vm_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct StorageManager {
    storage_root: PathBuf,
}

impl StorageManager {
    pub async fn new() -> Result<Self> {
        // Use dirs crate for secure home directory detection
        let storage_root = home_dir()
            .map(|h| h.join(".vortex").join("storage"))
            .unwrap_or_else(|| PathBuf::from("/tmp/vortex/storage"));

        std::fs::create_dir_all(&storage_root)?;

        Ok(Self { storage_root })
    }

    pub async fn create_volume(&self, name: String, size_bytes: u64) -> Result<Volume> {
        let id = uuid::Uuid::new_v4().to_string();
        let path = self.storage_root.join(&id);

        // Create empty volume file with secure permissions
        let file = std::fs::File::create(&path)?;
        #[cfg(unix)]
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        #[cfg(not(unix))]
        // On non-Unix systems, just set the length
        let _ = file;
        file.set_len(size_bytes)?;

        Ok(Volume {
            id,
            name,
            path,
            size_bytes,
            vm_id: None,
            created_at: chrono::Utc::now(),
        })
    }

    pub async fn attach_volume(&self, _volume_id: &str, _vm_id: &str) -> Result<()> {
        // In real implementation, would update volume metadata and notify backend
        Ok(())
    }

    pub async fn detach_volume(&self, _volume_id: &str) -> Result<()> {
        Ok(())
    }

    pub async fn delete_volume(&self, _volume_id: &str) -> Result<()> {
        // Delete volume file
        Ok(())
    }
}
