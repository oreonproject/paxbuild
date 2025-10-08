use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use tempfile::TempDir;
use crate::recipe::BuildRecipe;
use crate::source::SourceManager;

/// Package builder that creates .pax packages from recipes
pub struct PackageBuilder {
    temp_dir: TempDir,
    source_mgr: SourceManager,
}

impl PackageBuilder {
    /// Create a new package builder
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()
            .with_context(|| "Failed to create temporary directory")?;
        
        let source_mgr = SourceManager::new()?;
        
        Ok(PackageBuilder {
            temp_dir,
            source_mgr,
        })
    }

    /// Build a package from a recipe
    pub fn build(&self, recipe: &BuildRecipe) -> Result<PathBuf> {
        let package_paths = self.build_for_architectures(recipe, &recipe.arch)?;
        // For backward compatibility, return the first package path
        package_paths.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("No packages were built"))
    }

    /// Build a package for specific architectures
    pub fn build_for_architectures(&self, recipe: &BuildRecipe, architectures: &[String]) -> Result<Vec<PathBuf>> {
        println!("Building package: {} {} for architectures: {:?}",
                 recipe.name, recipe.version, architectures);

        // Validate recipe
        recipe.validate()?;

        if architectures.is_empty() {
            anyhow::bail!("No architectures specified for build");
        }

        // Download and extract source once (shared across architectures)
        let source_dir = self.source_mgr.download_and_extract(
            &recipe.source,
            recipe.hash.as_deref(),
        )?;

        // Build for each architecture
        let mut package_paths = Vec::new();
        for target_arch in architectures {
            println!("Building for architecture: {}", target_arch);

            // Run build script for specific architecture
            self.run_build_script_for_arch(recipe, &source_dir, target_arch)?;

            // Create package for specific architecture
            let package_path = self.create_package_for_arch(recipe, target_arch)?;
            println!("Package created: {}", package_path.display());
            package_paths.push(package_path);
        }

        println!("All architecture-specific packages built in temp directory");
        Ok(package_paths)
    }

    /// Run the build script for a specific architecture
    fn run_build_script_for_arch(&self, recipe: &BuildRecipe, source_dir: &Path, arch: &str) -> Result<()> {
        println!("Running build script for architecture: {}...", arch);

        let build_dir = self.temp_dir.path().join("build");
        let install_dir = self.temp_dir.path().join("install");

        fs::create_dir_all(&build_dir)
            .with_context(|| "Failed to create build directory")?;
        fs::create_dir_all(&install_dir)
            .with_context(|| "Failed to create install directory")?;

        let build_script = recipe.get_build_script();

        // Set up environment variables with target architecture
        let mut cmd = Command::new("bash");
        cmd.arg("-c")
            .arg(&build_script)
            .current_dir(source_dir)
            .env("PAX_BUILD_ROOT", &install_dir)
            .env("PAX_PACKAGE_NAME", &recipe.name)
            .env("PAX_PACKAGE_VERSION", &recipe.version)
            .env("PAX_ARCH", arch)
            .env("PAX_TARGET_ARCH", arch)
            .env("PAX_SOURCE_DIR", source_dir)
            .env("PAX_BUILD_DIR", &build_dir);

        let output = cmd.output()
            .with_context(|| format!("Failed to run build command for architecture {}", arch))?;

        if !output.status.success() {
            println!("Build output for {}:", arch);
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("Build errors for {}:", arch);
            println!("{}", String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("Build script failed for architecture {}", arch);
        }

        println!("Build completed successfully for architecture: {}", arch);
        Ok(())
    }

    /// Create the .pax package for a specific architecture
    fn create_package_for_arch(&self, recipe: &BuildRecipe, arch: &str) -> Result<PathBuf> {
        println!("Creating package for architecture: {}...", arch);

        let package_dir = self.temp_dir.path().join("package");
        fs::create_dir_all(&package_dir)
            .with_context(|| "Failed to create package directory")?;

        // Copy installed files to package directory
        let install_dir = self.temp_dir.path().join("install");
        if install_dir.exists() {
            self.copy_directory(&install_dir, &package_dir)?;
        }

        // Create package metadata file (not .paxmeta, but actual package metadata)
        let metadata_path = package_dir.join("metadata.yaml");
        let metadata_content = self.create_package_metadata_for_arch(recipe, arch)?;
        fs::write(&metadata_path, metadata_content)
            .with_context(|| "Failed to write metadata file")?;

        // Create the .pax package (zstd-compressed tarball) with architecture in filename
        let package_filename = recipe.package_filename_for_arch(arch);
        let package_path = self.temp_dir.path().join(package_filename);
        self.create_tarball(&package_dir, &package_path)?;

        Ok(package_path)
    }
    
    /// Create package metadata for the installed package for a specific architecture
    fn create_package_metadata_for_arch(&self, recipe: &BuildRecipe, arch: &str) -> Result<String> {
        use serde_yaml;

        #[derive(serde::Serialize)]
        struct PackageMetadata {
            name: String,
            version: String,
            description: String,
            arch: Vec<String>,
            dependencies: Vec<String>,
            runtime_dependencies: Vec<String>,
            provides: Vec<String>,
            conflicts: Vec<String>,
            install_script: Option<String>,
            uninstall_script: Option<String>,
            files: Vec<String>,
        }

        // List all files in the package
        let install_dir = self.temp_dir.path().join("install");
        let files = if install_dir.exists() {
            self.list_files_recursive(&install_dir)?
        } else {
            Vec::new()
        };

        let metadata = PackageMetadata {
            name: recipe.name.clone(),
            version: recipe.version.clone(),
            description: recipe.description.clone(),
            arch: vec![arch.to_string()], // Only include the target architecture
            dependencies: recipe.dependencies.clone(),
            runtime_dependencies: recipe.runtime_dependencies.clone(),
            provides: if recipe.provides.is_empty() {
                vec![recipe.name.clone()]
            } else {
                recipe.provides.clone()
            },
            conflicts: recipe.conflicts.clone(),
            install_script: recipe.install.clone(),
            uninstall_script: recipe.uninstall.clone(),
            files,
        };

        serde_yaml::to_string(&metadata)
            .with_context(|| "Failed to serialize package metadata")
    }
    
    /// List files recursively from a directory
    fn list_files_recursive(&self, dir: &Path) -> Result<Vec<String>> {
        let mut files = Vec::new();
        
        if !dir.exists() {
            return Ok(files);
        }
        
        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry.with_context(|| "Failed to read directory entry")?;
            if entry.file_type().is_file() {
                let relative_path = entry.path()
                    .strip_prefix(dir)
                    .with_context(|| "Failed to strip prefix")?
                    .to_string_lossy()
                    .to_string();
                files.push(relative_path);
            }
        }
        
        Ok(files)
    }

    /// Copy directory recursively
    fn copy_directory(&self, src: &Path, dst: &Path) -> Result<()> {
        if !src.exists() {
            return Ok(());
        }
        
        if src.is_file() {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| "Failed to create destination directory")?;
            }
            fs::copy(src, dst)
                .with_context(|| "Failed to copy file")?;
            return Ok(());
        }
        
        for entry in fs::read_dir(src)
            .with_context(|| format!("Failed to read directory: {}", src.display()))? {
            let entry = entry.with_context(|| "Failed to read directory entry")?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if src_path.is_dir() {
                fs::create_dir_all(&dst_path)
                    .with_context(|| "Failed to create destination directory")?;
                self.copy_directory(&src_path, &dst_path)?;
            } else {
                if let Some(parent) = dst_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| "Failed to create destination directory")?;
                }
                fs::copy(&src_path, &dst_path)
                    .with_context(|| "Failed to copy file")?;
            }
        }
        
        Ok(())
    }

    /// Create a zstd-compressed tarball
    fn create_tarball(&self, src_dir: &Path, dst_path: &Path) -> Result<()> {
        let output = Command::new("tar")
            .arg("-cf")
            .arg("-")
            .arg("-C")
            .arg(src_dir)
            .arg(".")
            .output()
            .with_context(|| "Failed to create tar archive")?;
        
        if !output.status.success() {
            anyhow::bail!("Failed to create tar archive");
        }
        
        // Compress with zstd
        let mut zstd_cmd = Command::new("zstd");
        zstd_cmd.arg("-c")
            .arg("-19") // High compression level
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped());
        
        let mut zstd_process = zstd_cmd.spawn()
            .with_context(|| "Failed to start zstd process")?;
        
        if let Some(stdin) = zstd_process.stdin.take() {
            std::io::Write::write_all(&mut std::io::BufWriter::new(stdin), &output.stdout)
                .with_context(|| "Failed to write to zstd stdin")?;
        }
        
        let zstd_output = zstd_process.wait_with_output()
            .with_context(|| "Failed to wait for zstd process")?;
        
        if !zstd_output.status.success() {
            anyhow::bail!("Failed to compress with zstd");
        }
        
        fs::write(dst_path, zstd_output.stdout)
            .with_context(|| format!("Failed to write compressed package: {}", dst_path.display()))?;
        
        Ok(())
    }

    /// Get the temporary directory path
    pub fn temp_dir(&self) -> &Path {
        self.temp_dir.path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_copy_directory() {
        let builder = PackageBuilder::new().unwrap();
        
        // Create test source directory
        let src_dir = builder.temp_dir().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("file1.txt"), "content1").unwrap();
        
        let sub_dir = src_dir.join("subdir");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(sub_dir.join("file2.txt"), "content2").unwrap();
        
        // Copy to destination
        let dst_dir = builder.temp_dir().join("dst");
        builder.copy_directory(&src_dir, &dst_dir).unwrap();
        
        // Verify copy
        assert!(dst_dir.join("file1.txt").exists());
        assert!(dst_dir.join("subdir").join("file2.txt").exists());
    }
}
