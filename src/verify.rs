use anyhow::Result;
use crate::package::PaxPackage;

/// Verify a .pax package
pub fn verify_package(package_path: &str, key_path: Option<&str>) -> Result<()> {
    println!("PAXBuild - Verifying package");
    println!("Package: {}", package_path);
    
    let mut package = PaxPackage::open(package_path)?;
    
    // Verify package integrity
    println!("Verifying package integrity...");
    package.verify()?;
    println!("Package integrity verified");
    
    // Load metadata
    let mut package = package;
    let metadata = package.load_metadata()?;
    println!("Package metadata:");
    println!("  Name: {}", metadata.name);
    println!("  Version: {}", metadata.version);
    println!("  Description: {}", metadata.description);
    
    // Verify signature if key provided
    if let Some(key) = key_path {
        println!("Verifying signature with key: {}", key);

        // Look for signature file (package.sig or package.signature)
        let signature_path = format!("{}.sig", package_path);
        let signature_path_alt = format!("{}.signature", package_path);

        let signature_path = if std::path::Path::new(&signature_path).exists() {
            signature_path
        } else if std::path::Path::new(&signature_path_alt).exists() {
            signature_path_alt
        } else {
            println!("Warning: No signature file found (expected {}.sig or {}.signature)", package_path, package_path);
            return Ok(());
        };

        // Read signature
        let signature = std::fs::read(&signature_path)
            .with_context(|| format!("Failed to read signature file: {}", signature_path))?;

        // Verify signature using crypto module
        let is_valid = crate::crypto::verify_package_signature(
            std::path::Path::new(package_path),
            &signature,
            std::path::Path::new(key),
        )?;

        if is_valid {
            println!("✓ Package signature is valid");
        } else {
            println!("✗ Package signature is invalid");
            anyhow::bail!("Package signature verification failed");
        }
    }
    
    // Calculate and display hash
    let hash = package.calculate_hash()?;
    println!("Package hash: {}", hash);
    
    // List files
    let files = package.list_files()?;
    println!("Package contains {} files", files.len());
    
    Ok(())
}
