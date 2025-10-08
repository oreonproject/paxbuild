# PAXBuild

PAXBuild is a package builder for the PAX package manager. It creates `.pax` packages from `.paxmeta` recipes.

## Installation

```bash
cargo build --release
sudo cp target/release/paxbuild /usr/local/bin/
```

### Build a Package

```bash
# Build from URL
paxbuild build https://example.com/package.paxmeta

# Build for specific architecture
paxbuild build package.paxmeta --arch x86_64

# Build for multiple architectures
paxbuild build package.paxmeta --arch x86_64 --arch aarch64

# Build with custom output path (for single architecture)
paxbuild build package.paxmeta --arch x86_64 --output /tmp/my-package-x86_64.pax

# Build with custom output directory (for multiple architectures)
paxbuild build package.paxmeta --arch x86_64 --arch aarch64 --output /tmp/packages/

# Verbose output
paxbuild build package.paxmeta --verbose
```

### Verify a Package

```bash
# Verify package integrity
paxbuild verify package.pax

# Verify with signature
paxbuild verify package.pax --key public.key
```

### Sign a Package

```bash
# Sign package
paxbuild sign package.pax --key private.key

# Sign with custom output
paxbuild sign package.pax --key private.key --output package.pax.sig
```

### Show Package Information

```bash
# Show package info
paxbuild info package.pax
```

### Extract Package Contents

```bash
# Extract to current directory
paxbuild extract package.pax

# Extract to specific directory
paxbuild extract package.pax --output /tmp/extracted
```

### Key Management

PAXBuild provides comprehensive key management functionality for cryptographic operations:

#### Generate Key Pair

```bash
# Generate a new Ed25519 key pair
paxbuild keys generate --private private.key --public public.key

# Generate with force overwrite
paxbuild keys generate --private private.key --public public.key --force
```

#### Key Information

```bash
# Show private key information
paxbuild keys info --key private.key --type private

# Show public key information
paxbuild keys info --key public.key --type public
```

#### List Keys

```bash
# List keys in current directory
paxbuild keys list

# List keys in specific directory
paxbuild keys list --directory ./keys/
```

#### Export Public Key

```bash
# Export public key from private key
paxbuild keys export --private private.key --public exported-public.key
```

#### Import Key

```bash
# Import a key from another location
paxbuild keys import --source backup.key --dest imported.key --type private
```

#### Backup Keys

```bash
# Backup all keys from directory
paxbuild keys backup --source ./keys/ --dest ./backup/
```

#### Key Types

- **Private keys**: Used for signing packages (32 bytes Ed25519)
- **Public keys**: Used for signature verification (32 bytes Ed25519)

All keys are stored as hexadecimal strings in `.key` files.

## Recipe Format (.paxmeta)

PAXBuild uses YAML recipe files to define how to build packages:

```yaml
name: package-name
version: 1.0.0
description: Package description
source: https://example.com/package-1.0.0.tar.gz
hash: sha256:abc123...  # Optional, auto-generated if missing

# Build configuration
build: |
  ./configure --prefix=/usr
  make -j$(nproc)
  make install DESTDIR=$PAX_BUILD_ROOT

# Dependencies
dependencies:
  - libc>=2.31
  - libssl>=1.1

runtime_dependencies:
  - libc.so.6
  - libssl.so.1.1

# Package metadata
provides:
  - package-name
  - package-bin

conflicts:
  - old-package

# Scripts
install: |
  ldconfig
  update-desktop-database

uninstall: |
  ldconfig
```

### Build env variables

The build script has access to these environment variables:

- `PAX_BUILD_ROOT`: Installation destination
- `PAX_PACKAGE_NAME`: Package name
- `PAX_PACKAGE_VERSION`: Package version
- `PAX_ARCH`: Target architecture
- `PAX_SOURCE_DIR`: Source directory
- `PAX_BUILD_DIR`: Build directory

## Multi-Architecture Support

PAXBuild supports building packages for multiple architectures in a single command:

```bash
# Build for specific architectures
paxbuild build package.paxmeta --arch x86_64 --arch aarch64

# Build for all architectures defined in recipe
paxbuild build package.paxmeta

# Supported architectures: x86_64, aarch64, armv7, i686, riscv64
```

### Package Naming

Multi-architecture packages use the format: `name-version-architecture.pax`

- `hello-world-1.0.0-x86_64.pax` (x86_64 architecture)
- `hello-world-1.0.0-aarch64.pax` (ARM64 architecture)

When building multiple architectures:
- Without `--output`: Packages are created in temp directory with proper names
- With `--output` for single architecture: Package copied to specified path
- With `--output` for multiple architectures: Output treated as directory, each architecture gets its own file

## Package Format (.pax)

PAX packages are zstd-compressed tarballs containing:

- `metadata.yaml`: Package metadata (YAML) with installation information
- Package files in standard Linux directory structure (usr/bin/, usr/lib/, etc.)
- Optional signature file

The `.pax` format is a compiled package ready for direct installation by PAX, not a local build recipe.

## Examples

### Simple Autotools Package

```yaml
name: htop
version: 3.2.1
description: Interactive process viewer
source: https://github.com/htop-dev/htop/releases/download/3.2.1/htop-3.2.1.tar.xz
```

### Custom Build Script

```yaml
name: myapp
version: 1.0.0
description: My custom application
source: https://github.com/user/myapp/archive/v1.0.0.tar.gz
build: |
  cargo build --release
  mkdir -p $PAX_BUILD_ROOT/usr/bin
  cp target/release/myapp $PAX_BUILD_ROOT/usr/bin/
```

### Package with Dependencies

```yaml
name: myapp
version: 1.0.0
description: My application
source: https://example.com/myapp-1.0.0.tar.gz
dependencies:
  - libc>=2.31
  - libssl>=1.1
runtime_dependencies:
  - libc.so.6
  - libssl.so.1.1
provides:
  - myapp
  - myapp-bin
```

## Integration with PAX itself

PAXBuild integrates with the PAX package manager's `compile` command:

```bash
# PAX will automatically use paxbuild
sudo pax compile https://github.com/user/project
sudo pax compile ./package.paxmeta
```

## Security and signing

- Packages are signed with Ed25519 signatures
- Source checksums are verified
- Build scripts run in isolated environment
- Temporary files are cleaned up automatically

## Dependencies

- Rust 1.70+
- tar
- zstd
- unzip (for zip archives)

## Help us out
feel free to contribute!