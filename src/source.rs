use anyhow::{Result, Context};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Manages source code download and extraction
pub struct SourceManager {
    temp_dir: TempDir,
}

impl SourceManager {
    /// Create a new source manager
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()
            .with_context(|| "Failed to create temporary directory")?;
        
        Ok(SourceManager { temp_dir })
    }

    /// Download and extract source code
    pub fn download_and_extract(&self, url: &str, expected_hash: Option<&str>) -> Result<PathBuf> {
        println!("Downloading source from: {}", url);
        
        // Download the source
        let source_file = self.download_source(url)?;
        
        // Verify hash if provided
        if let Some(expected) = expected_hash {
            self.verify_hash(&source_file, expected)?;
        }
        
        // Extract the source
        let extracted_dir = self.extract_source(&source_file)?;
        
        Ok(extracted_dir)
    }

    /// Download source file
    fn download_source(&self, url: &str) -> Result<PathBuf> {
        let filename = self.get_filename_from_url(url);
        let dest_path = self.temp_dir.path().join(&filename);
        
        let mut response = reqwest::blocking::get(url)
            .with_context(|| format!("Failed to download from: {}", url))?;
        
        if !response.status().is_success() {
            anyhow::bail!("HTTP error {}: {}", response.status(), url);
        }
        
        let mut file = fs::File::create(&dest_path)
            .with_context(|| format!("Failed to create file: {}", dest_path.display()))?;
        
        std::io::copy(&mut response, &mut file)
            .with_context(|| "Failed to write downloaded file")?;
        
        println!("Downloaded to: {}", dest_path.display());
        Ok(dest_path)
    }

    /// Extract source archive
    fn extract_source(&self, archive_path: &Path) -> Result<PathBuf> {
        let extract_dir = self.temp_dir.path().join("extracted");
        fs::create_dir_all(&extract_dir)
            .with_context(|| "Failed to create extract directory")?;
        
        println!("Extracting archive...");
        
        // Determine archive type and extract
        let filename = archive_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
        
        let success = if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
            self.extract_tar_gz(archive_path, &extract_dir)?
        } else if filename.ends_with(".tar.xz") {
            self.extract_tar_xz(archive_path, &extract_dir)?
        } else if filename.ends_with(".tar.bz2") {
            self.extract_tar_bz2(archive_path, &extract_dir)?
        } else if filename.ends_with(".zip") {
            self.extract_zip(archive_path, &extract_dir)?
        } else if filename.ends_with(".tar") {
            self.extract_tar(archive_path, &extract_dir)?
        } else {
            anyhow::bail!("Unsupported archive format: {}", filename);
        };
        
        if !success {
            anyhow::bail!("Failed to extract archive");
        }
        
        // Find the extracted directory (usually has package name)
        let extracted_package_dir = self.find_extracted_dir(&extract_dir)?;
        println!("Extracted to: {}", extracted_package_dir.display());
        
        Ok(extracted_package_dir)
    }

    /// Extract tar.gz archive
    fn extract_tar_gz(&self, archive_path: &Path, dest_dir: &Path) -> Result<bool> {
        let output = Command::new("tar")
            .arg("-xzf")
            .arg(archive_path)
            .arg("-C")
            .arg(dest_dir)
            .output()
            .with_context(|| "Failed to run tar command")?;
        
        Ok(output.status.success())
    }

    /// Extract tar.xz archive
    fn extract_tar_xz(&self, archive_path: &Path, dest_dir: &Path) -> Result<bool> {
        let output = Command::new("tar")
            .arg("-xJf")
            .arg(archive_path)
            .arg("-C")
            .arg(dest_dir)
            .output()
            .with_context(|| "Failed to run tar command")?;
        
        Ok(output.status.success())
    }

    /// Extract tar.bz2 archive
    fn extract_tar_bz2(&self, archive_path: &Path, dest_dir: &Path) -> Result<bool> {
        let output = Command::new("tar")
            .arg("-xjf")
            .arg(archive_path)
            .arg("-C")
            .arg(dest_dir)
            .output()
            .with_context(|| "Failed to run tar command")?;
        
        Ok(output.status.success())
    }

    /// Extract zip archive
    fn extract_zip(&self, archive_path: &Path, dest_dir: &Path) -> Result<bool> {
        let output = Command::new("unzip")
            .arg("-q")
            .arg(archive_path)
            .arg("-d")
            .arg(dest_dir)
            .output()
            .with_context(|| "Failed to run unzip command")?;
        
        Ok(output.status.success())
    }

    /// Extract tar archive
    fn extract_tar(&self, archive_path: &Path, dest_dir: &Path) -> Result<bool> {
        let output = Command::new("tar")
            .arg("-xf")
            .arg(archive_path)
            .arg("-C")
            .arg(dest_dir)
            .output()
            .with_context(|| "Failed to run tar command")?;
        
        Ok(output.status.success())
    }

    /// Find the extracted directory
    fn find_extracted_dir(&self, extract_dir: &Path) -> Result<PathBuf> {
        let entries: Vec<_> = fs::read_dir(extract_dir)
            .with_context(|| "Failed to read extract directory")?
            .filter_map(|entry| entry.ok())
            .collect();
        
        if entries.len() == 1 && entries[0].path().is_dir() {
            Ok(entries[0].path())
        } else {
            // Multiple entries or single file, return the extract directory
            Ok(extract_dir.to_path_buf())
        }
    }

    /// Verify file hash
    fn verify_hash(&self, file_path: &Path, expected_hash: &str) -> Result<()> {
        println!("Verifying hash...");
        
        let mut file = fs::File::open(file_path)
            .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
        
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)
            .with_context(|| "Failed to read file for hashing")?;
        
        let calculated_hash = hex::encode(hasher.finalize());
        let expected_clean = expected_hash.replace("sha256:", "");
        
        if calculated_hash != expected_clean {
            anyhow::bail!(
                "Hash mismatch! Expected: {}, Calculated: {}",
                expected_clean,
                calculated_hash
            );
        }
        
        println!("Hash verified: {}", calculated_hash);
        Ok(())
    }

    /// Get filename from URL
    fn get_filename_from_url(&self, url: &str) -> String {
        url.split('/')
            .last()
            .unwrap_or("source.tar.gz")
            .to_string()
    }

    /// Calculate SHA256 hash of a file
    pub fn calculate_hash(file_path: &Path) -> Result<String> {
        let mut file = fs::File::open(file_path)
            .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
        
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)
            .with_context(|| "Failed to read file for hashing")?;
        
        Ok(hex::encode(hasher.finalize()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_get_filename_from_url() {
        let manager = SourceManager::new().unwrap();
        assert_eq!(manager.get_filename_from_url("https://example.com/test.tar.gz"), "test.tar.gz");
        assert_eq!(manager.get_filename_from_url("https://example.com/path/to/file.zip"), "file.zip");
    }

    #[test]
    fn test_calculate_hash() {
        // Create a temporary file for testing
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, World!").unwrap();
        
        let hash = SourceManager::calculate_hash(&test_file).unwrap();
        // This is the SHA256 of "Hello, World!"
        assert_eq!(hash, "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f");
    }
}
