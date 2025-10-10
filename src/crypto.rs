use anyhow::{Result, Context};
use rand::RngCore;
use rand::rngs::OsRng;
use std::fs;
use std::path::Path;
use hex;


/// Generate a simple key pair for testing (no cryptographic signing)
pub fn generate_key_pair() -> Result<(Vec<u8>, Vec<u8>)> {
    let mut csprng = OsRng;
    let mut private_key = [0u8; 32];
    let mut public_key = [0u8; 32];

    csprng.fill_bytes(&mut private_key);
    csprng.fill_bytes(&mut public_key);

    Ok((private_key.to_vec(), public_key.to_vec()))
}

/// Save key pair to files
pub fn save_key_pair(
    private_key_path: &Path,
    public_key_path: &Path,
) -> Result<()> {
    let (private_key, public_key) = generate_key_pair()?;
    
    // Save private key
    fs::write(private_key_path, hex::encode(private_key))
        .with_context(|| format!("Failed to write private key: {}", private_key_path.display()))?;
    
    // Save public key
    fs::write(public_key_path, hex::encode(public_key))
        .with_context(|| format!("Failed to write public key: {}", public_key_path.display()))?;
    
    Ok(())
}

/// Load key pair from files
pub fn load_key_pair(
    private_key_path: &Path,
    public_key_path: &Path,
) -> Result<(Vec<u8>, Vec<u8>)> {
    // Read private key
    let private_key_hex = fs::read_to_string(private_key_path)
        .with_context(|| format!("Failed to read private key: {}", private_key_path.display()))?;
    
    let private_key = hex::decode(private_key_hex.trim())
        .with_context(|| "Failed to decode private key hex")?;
    
    // Read public key
    let public_key_hex = fs::read_to_string(public_key_path)
        .with_context(|| format!("Failed to read public key: {}", public_key_path.display()))?;
    
    let public_key = hex::decode(public_key_hex.trim())
        .with_context(|| "Failed to decode public key hex")?;
    
    Ok((private_key, public_key))
}

/// Get key fingerprint for identification
pub fn get_key_fingerprint(key_bytes: &[u8]) -> Result<String> {
    use sha2::{Digest, Sha256};

    if key_bytes.len() != 32 {
        anyhow::bail!("Invalid key length for fingerprint: {}", key_bytes.len());
    }

    let mut hasher = Sha256::new();
    hasher.update(key_bytes);
    let fingerprint = hasher.finalize();

    Ok(hex::encode(fingerprint))
}

/// Validate key format and return key type
pub fn validate_key(key_path: &Path) -> Result<String> {
    let key_data = fs::read_to_string(key_path)
        .with_context(|| format!("Failed to read key file: {}", key_path.display()))?;

    let key_bytes = hex::decode(key_data.trim())
        .with_context(|| format!("Failed to decode key hex in: {}", key_path.display()))?;

    if key_bytes.len() != 32 {
        anyhow::bail!("Invalid key length: expected 32 bytes, got {}", key_bytes.len());
    }

    // Simple validation - just check length and hex format
    Ok("generic".to_string())
}

/// Check if a key pair matches (basic file validation only)
pub fn validate_key_pair(private_key_path: &Path, public_key_path: &Path) -> Result<bool> {
    let (private_key, public_key) = load_key_pair(private_key_path, public_key_path)?;

    // Basic validation - just check both files exist and are readable
    Ok(private_key.len() == 32 && public_key.len() == 32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_key_generation() {
        let (private_key, public_key) = generate_key_pair().unwrap();
        assert_eq!(private_key.len(), 32);
        assert_eq!(public_key.len(), 32);
    }

    #[test]
    fn test_key_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let private_key_path = temp_dir.path().join("private.key");
        let public_key_path = temp_dir.path().join("public.key");
        
        // Save key pair
        save_key_pair(&private_key_path, &public_key_path).unwrap();
        
        // Load key pair
        let (loaded_private, loaded_public) = load_key_pair(&private_key_path, &public_key_path).unwrap();
        
        assert_eq!(loaded_private.len(), 32);
        assert_eq!(loaded_public.len(), 32);
    }

}
