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
        // TODO: Implement signature verification
        println!("Signature verification not yet implemented");
    }
    
    // Calculate and display hash
    let hash = package.calculate_hash()?;
    println!("Package hash: {}", hash);
    
    // List files
    let files = package.list_files()?;
    println!("Package contains {} files", files.len());
    
    Ok(())
}
