use anyhow::{Result, Context};
use crate::recipe::BuildRecipe;
use crate::builder::PackageBuilder;

/// Build a package from a recipe
pub fn build_package(recipe_path: &str, output_path: Option<&str>, architectures: &[String], verbose: bool) -> Result<()> {
    println!("PAXBuild - Building package from recipe");
    println!("Recipe: {}", recipe_path);
    
    if verbose {
        println!("Verbose mode enabled");
    }
    
    // Load recipe
    let recipe = if recipe_path.starts_with("http://") || recipe_path.starts_with("https://") {
        BuildRecipe::from_url(recipe_path)?
    } else {
        BuildRecipe::from_file(recipe_path)?
    };

    if verbose {
        println!("Loaded recipe:");
        println!("  Name: {}", recipe.name);
        println!("  Version: {}", recipe.version);
        println!("  Description: {}", recipe.description);
        println!("  Source: {}", recipe.source);
        if let Some(hash) = &recipe.hash {
            println!("  Hash: {}", hash);
        }
        println!("  Dependencies: {:?}", recipe.dependencies);
        println!("  Provides: {:?}", recipe.provides);
        println!("  Architectures: {:?}", recipe.arch);
    }

    // Determine target architectures
    let target_architectures = if architectures.is_empty() {
        recipe.arch.clone()
    } else {
        for arch in architectures {
            if !recipe.arch.contains(arch) {
                anyhow::bail!("Architecture '{}' is not supported by this recipe. Supported architectures: {:?}", arch, recipe.arch);
            }
        }
        architectures.to_vec()
    };

    if verbose {
        if target_architectures.len() == 1 {
            println!("Target architecture: {}", target_architectures[0]);
        } else {
            println!("Target architectures: {:?}", target_architectures);
        }
    }

    // Build package
    let builder = PackageBuilder::new()?;
    let package_paths = builder.build_for_architectures(&recipe, &target_architectures)?;

    // Handle output for multiple architectures
    if let Some(output) = output_path {
        if target_architectures.len() == 1 {
            // Single architecture - copy to specified output
            let package_path = &package_paths[0];
            std::fs::copy(package_path, output)
                .with_context(|| format!("Failed to copy package to: {}", output))?;
            println!("Package saved to: {}", output);
        } else {
            // Multiple architectures - output should be a directory
            let output_dir = std::path::Path::new(output);
            if !output_dir.exists() {
                std::fs::create_dir_all(output_dir)
                    .with_context(|| format!("Failed to create output directory: {}", output))?;
            }

            for (i, package_path) in package_paths.iter().enumerate() {
                let arch = &target_architectures[i];
                let filename = format!("{}-{}.pax", recipe.package_id(), arch);
                let dest_path = output_dir.join(filename);

                std::fs::copy(package_path, &dest_path)
                    .with_context(|| format!("Failed to copy package to: {}", dest_path.display()))?;
                println!("Package for {} saved to: {}", arch, dest_path.display());
            }
        }
    } else {
        // No output specified - packages are in temp directory with proper names
        for (i, package_path) in package_paths.iter().enumerate() {
            let arch = &target_architectures[i];
            println!("Package for {} built at: {}", arch, package_path.display());
        }
    }
    
    Ok(())
}
