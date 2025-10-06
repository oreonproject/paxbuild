use anyhow::Result;
use crate::package::PaxPackage;

/// Show information about a .pax package
pub fn show_info(package_path: &str) -> Result<()> {
    println!("PAXBuild - Package Information");
    println!("Package: {}", package_path);
    println!();
    
    let mut package = PaxPackage::open(package_path)?;
    
    // Get package info first
    let size = package.size()?;
    let hash = package.calculate_hash()?;
    
    // Load metadata
    let metadata = package.load_metadata()?;
    
    // Display package information
    println!("Package Information:");
    println!("  Name: {}", metadata.name);
    println!("  Version: {}", metadata.version);
    println!("  Description: {}", metadata.description);
    println!("  Architectures: {:?}", metadata.arch);
    
    if !metadata.dependencies.is_empty() {
        println!("  Dependencies: {:?}", metadata.dependencies);
    }
    
    if !metadata.runtime_dependencies.is_empty() {
        println!("  Runtime Dependencies: {:?}", metadata.runtime_dependencies);
    }
    
    if !metadata.provides.is_empty() {
        println!("  Provides: {:?}", metadata.provides);
    }
    
    if !metadata.conflicts.is_empty() {
        println!("  Conflicts: {:?}", metadata.conflicts);
    }
    
    if let Some(install) = &metadata.install_script {
        println!("  Install Script: {}", install);
    }
    
    if let Some(uninstall) = &metadata.uninstall_script {
        println!("  Uninstall Script: {}", uninstall);
    }
    
    println!();
    
    // Display package file information
    
    println!("Package File Information:");
    println!("  Size: {} bytes", size);
    println!("  Hash: {}", hash);
    
    // List files from metadata
    println!("  Files: {}", metadata.files.len());
    
    if metadata.files.len() <= 20 {
        println!("  File List:");
        for file in &metadata.files {
            println!("    {}", file);
        }
    } else {
        println!("  File List (showing first 20):");
        for file in metadata.files.iter().take(20) {
            println!("    {}", file);
        }
        println!("    ... and {} more files", metadata.files.len() - 20);
    }
    
    Ok(())
}
