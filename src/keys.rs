use anyhow::{Result, Context};
use ed25519_dalek::{SigningKey, VerifyingKey};
use std::fs;
use std::path::{Path, PathBuf};
use hex;

/// Generate a new Ed25519 key pair and save to files
pub fn generate_key_pair_cmd(
    private_key_path: &str,
    public_key_path: &str,
    force: bool,
) -> Result<()> {
    let private_path = Path::new(private_key_path);
    let public_path = Path::new(public_key_path);

    // Check if files already exist
    if !force && (private_path.exists() || public_path.exists()) {
        anyhow::bail!(
            "Key files already exist. Use --force to overwrite.\nPrivate: {}\nPublic: {}",
            private_path.display(),
            public_path.display()
        );
    }

    println!("PAXBuild - Generating new key pair");
    println!("Private key: {}", private_key_path);
    println!("Public key: {}", public_key_path);

    let (private_key, public_key) = crate::crypto::generate_key_pair()?;

    // Save private key
    fs::write(private_path, hex::encode(private_key))
        .with_context(|| format!("Failed to write private key: {}", private_path.display()))?;

    // Save public key
    fs::write(public_path, hex::encode(public_key))
        .with_context(|| format!("Failed to write public key: {}", public_path.display()))?;

    println!("Key pair generated successfully!");

    Ok(())
}

/// Show information about a key
pub fn show_key_info(key_path: &str, key_type: &str) -> Result<()> {
    let path = Path::new(key_path);

    if !path.exists() {
        anyhow::bail!("Key file not found: {}", path.display());
    }

    println!("PAXBuild - Key information");
    println!("Key file: {}", key_path);
    println!("Key type: {}", key_type);

    // Read and decode key
    let key_hex = fs::read_to_string(path)
        .with_context(|| format!("Failed to read key file: {}", path.display()))?;

    let key_bytes = hex::decode(key_hex.trim())
        .with_context(|| format!("Failed to decode {} key hex", key_type))?;

    if key_bytes.len() != 32 {
        anyhow::bail!("Invalid key length: expected 32 bytes, got {}", key_bytes.len());
    }

    println!("Key length: {} bytes", key_bytes.len());
    println!("Key (hex): {}", hex::encode(&key_bytes));

    // Show fingerprint
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&key_bytes);
    let fingerprint = hasher.finalize();
    println!("Fingerprint: {}", hex::encode(fingerprint));

    // Validate key format
    match key_type {
        "private" => {
            SigningKey::from_bytes(&key_bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Invalid private key format"))?);
            println!("Status: Valid private key");
        }
        "public" => {
            VerifyingKey::from_bytes(&key_bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Invalid public key format"))?);
            println!("Status: Valid public key");
        }
        _ => {
            println!("Status: Unknown key type");
        }
    }

    Ok(())
}

/// List available keys in a directory
pub fn list_keys(directory: &str) -> Result<()> {
    let dir_path = Path::new(directory);

    if !dir_path.exists() {
        anyhow::bail!("Directory not found: {}", dir_path.display());
    }

    if !dir_path.is_dir() {
        anyhow::bail!("Path is not a directory: {}", dir_path.display());
    }

    println!("PAXBuild - Available keys in {}", directory);

    let mut found_keys = false;

    // Look for .key files
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("key") {
            found_keys = true;

            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("<invalid filename>");

            // Try to determine key type from filename
            let key_type = if file_name.contains("private") || file_name.contains("priv") {
                "Private"
            } else if file_name.contains("public") || file_name.contains("pub") {
                "Public"
            } else {
                "Unknown"
            };

            println!("  {} - {}", key_type, file_name);

            // Show key info if possible
            if let Ok(key_hex) = fs::read_to_string(&path) {
                if let Ok(key_bytes) = hex::decode(key_hex.trim()) {
                    if key_bytes.len() == 32 {
                        use sha2::{Digest, Sha256};
                        let mut hasher = Sha256::new();
                        hasher.update(&key_bytes);
                        let fingerprint = hasher.finalize();
                        println!("    Fingerprint: {}...", &hex::encode(fingerprint)[..16]);
                    }
                }
            }
        }
    }

    if !found_keys {
        println!("  No key files found in this directory");
    }

    Ok(())
}

/// Export a public key from a key pair
pub fn export_public_key(private_key_path: &str, public_key_path: &str) -> Result<()> {
    let private_path = Path::new(private_key_path);

    if !private_path.exists() {
        anyhow::bail!("Private key file not found: {}", private_path.display());
    }

    println!("PAXBuild - Exporting public key");
    println!("Private key: {}", private_key_path);
    println!("Public key: {}", public_key_path);

    // Load private key
    let private_key_hex = fs::read_to_string(private_path)
        .with_context(|| format!("Failed to read private key: {}", private_path.display()))?;

    let private_key_bytes = hex::decode(private_key_hex.trim())
        .with_context(|| "Failed to decode private key hex")?;

    let signing_key = SigningKey::from_bytes(&private_key_bytes.try_into()
        .map_err(|_| anyhow::anyhow!("Invalid private key length"))?);

    let verifying_key = signing_key.verifying_key();
    let public_key_bytes = verifying_key.to_bytes();

    // Save public key
    fs::write(public_key_path, hex::encode(public_key_bytes))
        .with_context(|| format!("Failed to write public key: {}", public_key_path))?;

    println!("Public key exported successfully!");

    Ok(())
}

/// Import a key from another source
pub fn import_key(source_path: &str, dest_path: &str, key_type: &str) -> Result<()> {
    let source_path = Path::new(source_path);
    let dest_path = Path::new(dest_path);

    if !source_path.exists() {
        anyhow::bail!("Source key file not found: {}", source_path.display());
    }

    println!("PAXBuild - Importing key");
    println!("Source: {}", source_path.display());
    println!("Destination: {}", dest_path.display());
    println!("Key type: {}", key_type);

    // Read source key
    let key_data = fs::read(source_path)
        .with_context(|| format!("Failed to read source key: {}", source_path.display()))?;

    // Validate key format
    let key_hex = String::from_utf8(key_data)
        .with_context(|| "Key file is not valid UTF-8")?;

    let key_bytes = hex::decode(key_hex.trim())
        .with_context(|| "Failed to decode key hex")?;

    if key_bytes.len() != 32 {
        anyhow::bail!("Invalid key length: expected 32 bytes, got {}", key_bytes.len());
    }

    // Validate key format based on type
    match key_type {
        "private" => {
            SigningKey::from_bytes(&key_bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Invalid private key format"))?);
        }
        "public" => {
            VerifyingKey::from_bytes(&key_bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Invalid public key format"))?);
        }
        _ => {
            anyhow::bail!("Invalid key type: {}", key_type);
        }
    }

    // Write to destination
    fs::write(dest_path, hex::encode(key_bytes))
        .with_context(|| format!("Failed to write key to: {}", dest_path.display()))?;

    println!("Key imported successfully!");

    Ok(())
}

/// Backup a key pair to a backup directory
pub fn backup_keys(key_dir: &str, backup_dir: &str) -> Result<()> {
    let key_path = Path::new(key_dir);
    let backup_path = Path::new(backup_dir);

    if !key_path.exists() {
        anyhow::bail!("Key directory not found: {}", key_path.display());
    }

    // Create backup directory if it doesn't exist
    fs::create_dir_all(backup_path)
        .with_context(|| format!("Failed to create backup directory: {}", backup_path.display()))?;

    println!("PAXBuild - Backing up keys");
    println!("Source directory: {}", key_path.display());
    println!("Backup directory: {}", backup_path.display());

    let mut backed_up = 0;

    // Copy all .key files
    for entry in fs::read_dir(key_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("key") {
            let file_name = path.file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

            let backup_file = backup_path.join(file_name);
            fs::copy(&path, &backup_file)
                .with_context(|| format!("Failed to backup key: {}", path.display()))?;

            backed_up += 1;
            println!("  Backed up: {}", file_name.to_string_lossy());
        }
    }

    println!("Backup completed! {} key files backed up.", backed_up);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_key_pair() {
        let temp_dir = TempDir::new().unwrap();
        let private_path = temp_dir.path().join("test_private.key");
        let public_path = temp_dir.path().join("test_public.key");

        generate_key_pair_cmd(
            private_path.to_str().unwrap(),
            public_path.to_str().unwrap(),
            false,
        ).unwrap();

        // Verify files were created
        assert!(private_path.exists());
        assert!(public_path.exists());

        // Verify file contents
        let private_content = fs::read_to_string(&private_path).unwrap();
        let public_content = fs::read_to_string(&public_path).unwrap();

        assert!(hex::decode(&private_content.trim()).is_ok());
        assert!(hex::decode(&public_content.trim()).is_ok());
    }

    #[test]
    fn test_show_key_info() {
        let temp_dir = TempDir::new().unwrap();
        let private_path = temp_dir.path().join("test_private.key");
        let public_path = temp_dir.path().join("test_public.key");

        // Generate key pair first
        generate_key_pair_cmd(
            private_path.to_str().unwrap(),
            public_path.to_str().unwrap(),
            false,
        ).unwrap();

        // Test showing key info
        show_key_info(private_path.to_str().unwrap(), "private").unwrap();
        show_key_info(public_path.to_str().unwrap(), "public").unwrap();
    }

    #[test]
    fn test_export_public_key() {
        let temp_dir = TempDir::new().unwrap();
        let private_path = temp_dir.path().join("test_private.key");
        let public_path = temp_dir.path().join("test_public.key");
        let exported_path = temp_dir.path().join("exported_public.key");

        // Generate key pair first
        generate_key_pair_cmd(
            private_path.to_str().unwrap(),
            public_path.to_str().unwrap(),
            false,
        ).unwrap();

        // Export public key
        export_public_key(
            private_path.to_str().unwrap(),
            exported_path.to_str().unwrap(),
        ).unwrap();

        // Verify exported file exists and has correct content
        assert!(exported_path.exists());

        let original_public = fs::read_to_string(&public_path).unwrap();
        let exported_public = fs::read_to_string(&exported_path).unwrap();

        assert_eq!(original_public, exported_public);
    }

    #[test]
    fn test_import_key() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.key");
        let dest_path = temp_dir.path().join("dest.key");

        // Create a test key
        let test_key = vec![1u8; 32];
        fs::write(&source_path, hex::encode(test_key)).unwrap();

        // Import the key
        import_key(
            source_path.to_str().unwrap(),
            dest_path.to_str().unwrap(),
            "public",
        ).unwrap();

        // Verify import worked
        assert!(dest_path.exists());

        let imported_content = fs::read_to_string(&dest_path).unwrap();
        let source_content = fs::read_to_string(&source_path).unwrap();

        assert_eq!(imported_content, source_content);
    }
}
