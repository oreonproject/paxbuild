use anyhow::Result;
use std::path::Path;
use crate::package::PaxPackage;

/// Extract contents of a .pax package
pub fn extract_package(package_path: &str, output_path: Option<&str>) -> Result<()> {
    println!("PAXBuild - Extracting package");
    println!("Package: {}", package_path);
    
    let package = PaxPackage::open(package_path)?;
    
    // Determine output directory
    let output_dir = if let Some(output) = output_path {
        Path::new(output).to_path_buf()
    } else {
        // Default to package name without extension
        let package_name = package.filename()
            .unwrap_or("extracted")
            .trim_end_matches(".pax");
        Path::new(package_name).to_path_buf()
    };
    
    println!("Extracting to: {}", output_dir.display());
    
    // Extract package
    package.extract_to(&output_dir)?;
    
    println!("Package extracted successfully");
    
    // List extracted files
    let files = package.list_files()?;
    println!("Extracted {} files", files.len());
    
    Ok(())
}
