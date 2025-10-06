use anyhow::{Result, Context};
use crate::recipe::BuildRecipe;
use crate::builder::PackageBuilder;

/// Build a package from a recipe
pub fn build_package(recipe_path: &str, output_path: Option<&str>, verbose: bool) -> Result<()> {
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
    }
    
    // Build package
    let builder = PackageBuilder::new()?;
    let package_path = builder.build(&recipe)?;
    
    // Move to output location if specified
    if let Some(output) = output_path {
        std::fs::copy(&package_path, output)
            .with_context(|| format!("Failed to copy package to: {}", output))?;
        println!("Package saved to: {}", output);
    } else {
        println!("Package built at: {}", package_path.display());
    }
    
    Ok(())
}
