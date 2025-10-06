use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Package metadata for installed packages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub arch: Vec<String>,
    pub dependencies: Vec<String>,
    pub runtime_dependencies: Vec<String>,
    pub provides: Vec<String>,
    pub conflicts: Vec<String>,
    pub install_script: Option<String>,
    pub uninstall_script: Option<String>,
    pub files: Vec<String>,
}

/// Represents a .pax package
pub struct PaxPackage {
    path: PathBuf,
    metadata: Option<PackageMetadata>,
}

impl PaxPackage {
    /// Open a .pax package file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        if !path.exists() {
            anyhow::bail!("Package file does not exist: {}", path.display());
        }
        
        Ok(PaxPackage {
            path,
            metadata: None,
        })
    }

    /// Load metadata from the package
    pub fn load_metadata(&mut self) -> Result<&PackageMetadata> {
        if self.metadata.is_some() {
            return Ok(self.metadata.as_ref().unwrap());
        }
        
        // Extract .paxmeta from the package
        let temp_dir = TempDir::new()
            .with_context(|| "Failed to create temporary directory")?;
        
        let extract_dir = temp_dir.path().join("extract");
        fs::create_dir_all(&extract_dir)
            .with_context(|| "Failed to create extract directory")?;
        
        // Decompress and extract
        self.extract_to(&extract_dir)?;
        
        // Find and read metadata.yaml
        let metadata_path = extract_dir.join("metadata.yaml");
        if !metadata_path.exists() {
            anyhow::bail!("metadata.yaml not found in package");
        }
        
        let contents = fs::read_to_string(&metadata_path)
            .with_context(|| "Failed to read metadata.yaml file")?;
        
        let metadata = self.parse_package_metadata(&contents)?;
        self.metadata = Some(metadata);
        
        Ok(self.metadata.as_ref().unwrap())
    }

    /// Extract package contents to a directory
    pub fn extract_to(&self, dest_dir: &Path) -> Result<()> {
        fs::create_dir_all(dest_dir)
            .with_context(|| "Failed to create destination directory")?;
        
        // Decompress with zstd and extract with tar
        let zstd_output = Command::new("zstd")
            .arg("-dc")
            .arg(&self.path)
            .output()
            .with_context(|| "Failed to decompress package")?;
        
        if !zstd_output.status.success() {
            anyhow::bail!("Failed to decompress package");
        }
        
        let mut tar_process = Command::new("tar")
            .arg("-xf")
            .arg("-")
            .arg("-C")
            .arg(dest_dir)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .with_context(|| "Failed to start tar process")?;
        
        if let Some(stdin) = tar_process.stdin.take() {
            std::io::Write::write_all(&mut std::io::BufWriter::new(stdin), &zstd_output.stdout)
                .with_context(|| "Failed to write to tar stdin")?;
        }
        
        let tar_output = tar_process.wait_with_output()
            .with_context(|| "Failed to wait for tar process")?;
        
        if !tar_output.status.success() {
            anyhow::bail!("Failed to extract package");
        }
        
        Ok(())
    }

    /// Get package file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get package filename
    pub fn filename(&self) -> Option<&str> {
        self.path.file_name()?.to_str()
    }

    /// Get package size
    pub fn size(&self) -> Result<u64> {
        let metadata = fs::metadata(&self.path)
            .with_context(|| format!("Failed to get metadata for: {}", self.path.display()))?;
        Ok(metadata.len())
    }

    /// Calculate SHA256 hash of the package
    pub fn calculate_hash(&self) -> Result<String> {
        use sha2::{Sha256, Digest};
        use hex;
        
        let mut file = fs::File::open(&self.path)
            .with_context(|| format!("Failed to open package: {}", self.path.display()))?;
        
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)
            .with_context(|| "Failed to read package for hashing")?;
        
        Ok(hex::encode(hasher.finalize()))
    }

    /// List files in the package
    pub fn list_files(&self) -> Result<Vec<PathBuf>> {
        let temp_dir = TempDir::new()
            .with_context(|| "Failed to create temporary directory")?;
        
        let extract_dir = temp_dir.path().join("extract");
        self.extract_to(&extract_dir)?;
        
        let mut files = Vec::new();
        self.collect_files(&extract_dir, &mut files)?;
        
        Ok(files)
    }

    /// Recursively collect files from a directory
    fn collect_files(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory: {}", dir.display()))? {
            let entry = entry.with_context(|| "Failed to read directory entry")?;
            let path = entry.path();
            
            if path.is_dir() {
                self.collect_files(&path, files)?;
            } else {
                files.push(path);
            }
        }
        
        Ok(())
    }

    /// Verify package integrity
    pub fn verify(&mut self) -> Result<()> {
        // Try to extract and read metadata
        self.load_metadata()?;
        
        // Try to list files
        self.list_files()?;
        
        Ok(())
    }
    
    /// Parse package metadata from YAML
    fn parse_package_metadata(&self, yaml: &str) -> Result<PackageMetadata> {
        serde_yaml::from_str(yaml)
            .with_context(|| "Failed to parse package metadata")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_package_operations() {
        // This test would require a real .pax package file
        // For now, just test that we can create a PaxPackage struct
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.pax");
        fs::write(&test_file, "test content").unwrap();
        
        let package = PaxPackage::open(&test_file).unwrap();
        assert_eq!(package.path(), test_file);
        assert_eq!(package.filename(), Some("test.pax"));
    }
}
