use anyhow::{Result, Context};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use std::fs;
use std::path::Path;
use hex;

/// Sign a .pax package with Ed25519
pub fn sign_package(package_path: &Path, private_key_path: &Path) -> Result<Vec<u8>> {
    // Read private key
    let key_data = fs::read(private_key_path)
        .with_context(|| format!("Failed to read private key: {}", private_key_path.display()))?;
    
    let key_data = hex::decode(&key_data)
        .with_context(|| "Failed to decode private key hex")?;
    
    let signing_key = SigningKey::from_bytes(&key_data[..32].try_into()
        .with_context(|| "Invalid private key length")?);
    
    // Read package file
    let package_data = fs::read(package_path)
        .with_context(|| format!("Failed to read package: {}", package_path.display()))?;
    
    // Sign the package
    let signature = signing_key.sign(&package_data);
    
    Ok(signature.to_bytes().to_vec())
}

/// Verify a .pax package signature
pub fn verify_package_signature(
    package_path: &Path,
    signature: &[u8],
    public_key_path: &Path,
) -> Result<bool> {
    // Read public key
    let key_data = fs::read(public_key_path)
        .with_context(|| format!("Failed to read public key: {}", public_key_path.display()))?;
    
    let key_data = hex::decode(&key_data)
        .with_context(|| "Failed to decode public key hex")?;
    
    let verifying_key = VerifyingKey::from_bytes(&key_data[..32].try_into()
        .with_context(|| "Invalid public key length")?)?;
    
    // Read package file
    let package_data = fs::read(package_path)
        .with_context(|| format!("Failed to read package: {}", package_path.display()))?;
    
    // Verify signature
    let signature = Signature::from_bytes(signature.try_into()
        .with_context(|| "Invalid signature length")?);
    
    match verifying_key.verify(&package_data, &signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Generate a new Ed25519 key pair
pub fn generate_key_pair() -> Result<(Vec<u8>, Vec<u8>)> {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    
    Ok((
        signing_key.to_bytes().to_vec(),
        verifying_key.to_bytes().to_vec(),
    ))
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

    // Try to parse as both key types to determine which one it is
    if SigningKey::from_bytes(&key_bytes.try_into().unwrap_or([0u8; 32])).to_bytes() == key_bytes {
        Ok("private".to_string())
    } else if VerifyingKey::from_bytes(&key_bytes.try_into().unwrap_or([0u8; 32])).to_bytes() == key_bytes {
        Ok("public".to_string())
    } else {
        anyhow::bail!("Key data does not match either private or public key format");
    }
}

/// Check if a key pair matches (public key derived from private key)
pub fn validate_key_pair(private_key_path: &Path, public_key_path: &Path) -> Result<bool> {
    let (private_key, public_key) = load_key_pair(private_key_path, public_key_path)?;

    let signing_key = SigningKey::from_bytes(&private_key.try_into()
        .map_err(|_| anyhow::anyhow!("Invalid private key length"))?);

    let derived_public_key = signing_key.verifying_key().to_bytes();

    Ok(derived_public_key.as_ref() == public_key.as_slice())
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

    #[test]
    fn test_sign_verify() {
        let temp_dir = TempDir::new().unwrap();
        let private_key_path = temp_dir.path().join("private.key");
        let public_key_path = temp_dir.path().join("public.key");
        let test_file = temp_dir.path().join("test.pax");
        
        // Generate key pair
        save_key_pair(&private_key_path, &public_key_path).unwrap();
        
        // Create test file
        fs::write(&test_file, "test package content").unwrap();
        
        // Sign the file
        let signature = sign_package(&test_file, &private_key_path).unwrap();
        
        // Verify the signature
        let is_valid = verify_package_signature(&test_file, &signature, &public_key_path).unwrap();
        assert!(is_valid);
        
        // Test with wrong signature
        let wrong_signature = vec![0u8; 64];
        let is_valid = verify_package_signature(&test_file, &wrong_signature, &public_key_path).unwrap();
        assert!(!is_valid);
    }
}
