use anyhow::{Result, Context};
use std::path::Path;
use crate::crypto::sign_package;

/// Sign a .pax package
pub fn sign_package_cmd(package_path: &str, key_path: &str, output_path: Option<&str>) -> Result<()> {
    println!("PAXBuild - Signing package");
    println!("Package: {}", package_path);
    println!("Key: {}", key_path);
    
    let signature = sign_package(
        Path::new(package_path),
        Path::new(key_path),
    )?;
    
    // Save signature to file
    let signature_path = if let Some(output) = output_path {
        output.to_string()
    } else {
        format!("{}.sig", package_path)
    };
    
    std::fs::write(&signature_path, &signature)
        .with_context(|| format!("Failed to write signature to: {}", signature_path))?;
    
    println!("Signature saved to: {}", signature_path);
    
    // Display signature as hex
    use hex;
    println!("Signature: {}", hex::encode(&signature));
    
    Ok(())
}
