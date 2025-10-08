use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

/// Build recipe format (.paxmeta)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRecipe {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package description
    pub description: String,
    /// Source URL (tarball, git repo, etc.)
    pub source: String,
    /// SHA256 checksum (optional, auto-generated if missing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Target architectures (defaults to x86_64, aarch64)
    #[serde(default = "default_arch")]
    pub arch: Vec<String>,
    /// Build dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Runtime dependencies
    #[serde(default)]
    pub runtime_dependencies: Vec<String>,
    /// What this package provides
    #[serde(default)]
    pub provides: Vec<String>,
    /// Packages this conflicts with
    #[serde(default)]
    pub conflicts: Vec<String>,
    /// Build script (runs in extracted source directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
    /// Post-install script (runs after installation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install: Option<String>,
    /// Post-uninstall script (runs before removal)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uninstall: Option<String>,
}

fn default_arch() -> Vec<String> {
    vec!["x86_64".to_string(), "aarch64".to_string()]
}

impl BuildRecipe {
    /// Load recipe from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read recipe file: {}", path.display()))?;
        
        Self::from_yaml(&contents)
    }

    /// Load recipe from a URL
    pub fn from_url(url: &str) -> Result<Self> {
        let response = reqwest::blocking::get(url)
            .with_context(|| format!("Failed to download recipe from: {}", url))?;
        
        if !response.status().is_success() {
            anyhow::bail!("HTTP error {}: {}", response.status(), url);
        }
        
        let contents = response.text()
            .with_context(|| format!("Failed to read response from: {}", url))?;
        
        Self::from_yaml(&contents)
    }

    /// Parse recipe from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml)
            .with_context(|| "Failed to parse recipe YAML")
    }

    /// Convert recipe to YAML string
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .with_context(|| "Failed to serialize recipe to YAML")
    }

    /// Get the default build script for autotools packages
    pub fn default_build_script() -> String {
        "./configure --prefix=/usr && make -j$(nproc) && make install DESTDIR=$PAX_BUILD_ROOT".to_string()
    }

    /// Get the build script, using default if none specified
    pub fn get_build_script(&self) -> String {
        self.build.clone().unwrap_or_else(Self::default_build_script)
    }

    /// Validate the recipe
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            anyhow::bail!("Package name cannot be empty");
        }
        if self.version.is_empty() {
            anyhow::bail!("Package version cannot be empty");
        }
        if self.description.is_empty() {
            anyhow::bail!("Package description cannot be empty");
        }
        if self.source.is_empty() {
            anyhow::bail!("Package source cannot be empty");
        }

        // Validate package name format
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            anyhow::bail!("Package name contains invalid characters. Only alphanumeric, dash, and underscore are allowed");
        }

        // Validate version format (basic check)
        if !self.version.chars().any(|c| c.is_numeric()) {
            anyhow::bail!("Package version must contain at least one number");
        }

        // Validate architectures
        Self::validate_architectures(&self.arch)?;

        Ok(())
    }

    /// Get package identifier (name-version)
    pub fn package_id(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }

    /// Get package filename
    pub fn package_filename(&self) -> String {
        format!("{}.pax", self.package_id())
    }

    /// Get package filename for a specific architecture
    pub fn package_filename_for_arch(&self, arch: &str) -> String {
        format!("{}-{}.pax", self.package_id(), arch)
    }

    /// Parse package name-version-arch.pax format
    pub fn parse_package_filename(filename: &str) -> Option<(String, String, String)> {
        if !filename.ends_with(".pax") {
            return None;
        }

        let name_without_ext = filename.trim_end_matches(".pax");
        let parts: Vec<&str> = name_without_ext.split('-').collect();

        if parts.len() < 3 {
            return None;
        }

        // Find the last part that could be an architecture (single alphanumeric word)
        let mut arch_index = None;
        for (i, part) in parts.iter().enumerate().rev() {
            // Check if this part looks like an architecture (alphanumeric + underscore, no dots, reasonable length)
            if part.chars().all(|c| c.is_alphanumeric() || c == '_') && !part.contains('.') && part.len() <= 20 {
                // Additional check: should be a known architecture
                if Self::is_architecture_supported(part) {
                    arch_index = Some(i);
                    break;
                }
            }
        }

        let arch_index = arch_index?;

        if arch_index == 0 {
            return None;
        }

        // Find where the version starts - work backwards from architecture
        // The version should be the rightmost part that looks like a version (contains dots or numbers)
        let mut version_start = arch_index;
        for (i, part) in parts.iter().enumerate().rev().skip(1) {
            if part.contains('.') || part.chars().all(|c| c.is_numeric()) {
                version_start = i; // i is the actual index since we're going backwards
                break;
            }
        }

        let name_parts = &parts[0..version_start];
        let name = name_parts.join("-");
        let version_parts = &parts[version_start..arch_index];
        let version = version_parts.join("-");
        let arch = parts[arch_index].to_string();

        // Validate that version contains at least one number
        if !version.chars().any(|c| c.is_numeric()) {
            return None;
        }

        Some((name, version, arch))
    }


    /// Validate architecture names
    pub fn validate_architectures(archs: &[String]) -> Result<()> {
        let valid_archs = ["x86_64", "aarch64", "armv7", "i686", "riscv64"];

        for arch in archs {
            if !valid_archs.contains(&arch.as_str()) {
                anyhow::bail!("Invalid architecture: {}. Valid architectures are: {:?}", arch, valid_archs);
            }
        }

        Ok(())
    }

    /// Get the current system architecture
    pub fn current_architecture() -> String {
        std::env::consts::ARCH.to_string()
    }

    /// Check if current system supports the given architecture
    pub fn is_architecture_supported(arch: &str) -> bool {
        let valid_archs = ["x86_64", "aarch64", "armv7", "i686", "riscv64"];
        valid_archs.contains(&arch)
    }

    /// Get compatible architectures for current system
    pub fn get_compatible_architectures() -> Vec<String> {
        let current = Self::current_architecture();
        match current.as_str() {
            "x86_64" => vec!["x86_64".to_string(), "i686".to_string()],
            "aarch64" => vec!["aarch64".to_string()],
            "armv7" => vec!["armv7".to_string()],
            _ => vec![current],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_parsing() {
        let yaml = r#"
name: test-package
version: 1.0.0
description: A test package
source: https://example.com/test-1.0.0.tar.gz
dependencies:
  - libc>=2.31
build: |
  make && make install DESTDIR=$PAX_BUILD_ROOT
"#;

        let recipe = BuildRecipe::from_yaml(yaml).unwrap();
        assert_eq!(recipe.name, "test-package");
        assert_eq!(recipe.version, "1.0.0");
        assert_eq!(recipe.dependencies.len(), 1);
        assert!(recipe.build.is_some());
    }

    #[test]
    fn test_recipe_validation() {
        let mut recipe = BuildRecipe {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            source: "https://example.com/test.tar.gz".to_string(),
            hash: None,
            arch: default_arch(),
            dependencies: vec![],
            runtime_dependencies: vec![],
            provides: vec![],
            conflicts: vec![],
            build: None,
            install: None,
            uninstall: None,
        };

        assert!(recipe.validate().is_ok());

        // Test invalid name
        recipe.name = "".to_string();
        assert!(recipe.validate().is_err());

        // Test invalid version
        recipe.name = "test".to_string();
        recipe.version = "".to_string();
        assert!(recipe.validate().is_err());
    }

    #[test]
    fn test_package_id() {
        let recipe = BuildRecipe {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            source: "https://example.com/test.tar.gz".to_string(),
            hash: None,
            arch: default_arch(),
            dependencies: vec![],
            runtime_dependencies: vec![],
            provides: vec![],
            conflicts: vec![],
            build: None,
            install: None,
            uninstall: None,
        };

        assert_eq!(recipe.package_id(), "test-package-1.0.0");
        assert_eq!(recipe.package_filename(), "test-package-1.0.0.pax");
    }

    #[test]
    fn test_package_filename_for_arch() {
        let recipe = BuildRecipe {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            source: "https://example.com/test.tar.gz".to_string(),
            hash: None,
            arch: default_arch(),
            dependencies: vec![],
            runtime_dependencies: vec![],
            provides: vec![],
            conflicts: vec![],
            build: None,
            install: None,
            uninstall: None,
        };

        assert_eq!(recipe.package_filename_for_arch("x86_64"), "test-package-1.0.0-x86_64.pax");
        assert_eq!(recipe.package_filename_for_arch("aarch64"), "test-package-1.0.0-aarch64.pax");
    }

    #[test]
    fn test_parse_package_filename() {
        // Test normal case
        assert_eq!(
            BuildRecipe::parse_package_filename("hello-world-1.0.0-x86_64.pax"),
            Some(("hello-world".to_string(), "1.0.0".to_string(), "x86_64".to_string()))
        );

        // Test version with multiple parts
        assert_eq!(
            BuildRecipe::parse_package_filename("my-package-2.1.3-aarch64.pax"),
            Some(("my-package".to_string(), "2.1.3".to_string(), "aarch64".to_string()))
        );

        // Test complex version
        assert_eq!(
            BuildRecipe::parse_package_filename("test-app-1.2.3-beta1-x86_64.pax"),
            Some(("test-app".to_string(), "1.2.3-beta1".to_string(), "x86_64".to_string()))
        );

        // Test invalid cases
        assert_eq!(BuildRecipe::parse_package_filename("invalid.pax"), None);
        assert_eq!(BuildRecipe::parse_package_filename("no-version-arch.pax"), None);
        assert_eq!(BuildRecipe::parse_package_filename("hello-world-1.0.0.pax"), None); // No arch
        assert_eq!(BuildRecipe::parse_package_filename("hello-world-x86_64.pax"), None); // No version
    }

    #[test]
    fn test_validate_architectures() {
        assert!(BuildRecipe::validate_architectures(&["x86_64".to_string(), "aarch64".to_string()]).is_ok());
        assert!(BuildRecipe::validate_architectures(&["armv7".to_string()]).is_ok());
        assert!(BuildRecipe::validate_architectures(&["i686".to_string(), "riscv64".to_string()]).is_ok());
        assert!(BuildRecipe::validate_architectures(&["invalid-arch".to_string()]).is_err());
    }
}
