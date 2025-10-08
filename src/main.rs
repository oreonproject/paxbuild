use clap::{Parser, Subcommand};
use paxbuild::{build, verify, sign, info, extract, keys};

#[derive(Parser)]
#[command(name = "paxbuild")]
#[command(about = "PAX package builder - builds .pax packages from .paxmeta recipes")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a .pax package from a .paxmeta recipe
    Build {
        /// Path to .paxmeta recipe file or URL
        recipe: String,
        /// Output path for the generated .pax package
        #[arg(short, long)]
        output: Option<String>,
        /// Target architecture(s) - can specify multiple (if not specified, builds for all architectures in recipe)
        #[arg(short, long)]
        arch: Vec<String>,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Verify a .pax package signature and checksum
    Verify {
        /// Path to .pax package file
        package: String,
        /// Public key file for verification
        #[arg(short, long)]
        key: Option<String>,
    },
    /// Sign a .pax package
    Sign {
        /// Path to .pax package file
        package: String,
        /// Private key file for signing
        #[arg(short, long)]
        key: String,
        /// Output path for signed package
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show information about a .pax package
    Info {
        /// Path to .pax package file
        package: String,
    },
    /// Extract contents of a .pax package
    Extract {
        /// Path to .pax package file
        package: String,
        /// Output directory for extracted contents
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Manage cryptographic keys
    Keys {
        #[command(subcommand)]
        command: KeyCommands,
    },
}

#[derive(Subcommand)]
enum KeyCommands {
    /// Generate a new Ed25519 key pair
    Generate {
        /// Path for the private key file
        #[arg(short, long)]
        private: String,
        /// Path for the public key file
        #[arg(short, long)]
        public: String,
        /// Force overwrite existing files
        #[arg(short, long)]
        force: bool,
    },
    /// Show information about a key
    Info {
        /// Path to the key file
        key: String,
        /// Type of key (private or public)
        #[arg(short, long)]
        type_: String,
    },
    /// List available keys in a directory
    List {
        /// Directory to search for key files
        #[arg(default_value = ".")]
        directory: String,
    },
    /// Export public key from a private key
    Export {
        /// Path to the private key file
        #[arg(short, long)]
        private: String,
        /// Path for the exported public key file
        #[arg(short, long)]
        public: String,
    },
    /// Import a key from another location
    Import {
        /// Source key file path
        source: String,
        /// Destination key file path
        dest: String,
        /// Type of key (private or public)
        #[arg(short, long)]
        type_: String,
    },
    /// Backup keys to a backup directory
    Backup {
        /// Source directory containing keys
        source: String,
        /// Backup directory destination
        dest: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { recipe, output, arch, verbose } => {
            build::build_package(&recipe, output.as_deref(), &arch, verbose)?;
        }
        Commands::Verify { package, key } => {
            verify::verify_package(&package, key.as_deref())?;
        }
        Commands::Sign { package, key, output } => {
            sign::sign_package_cmd(&package, &key, output.as_deref())?;
        }
        Commands::Info { package } => {
            info::show_info(&package)?;
        }
        Commands::Extract { package, output } => {
            extract::extract_package(&package, output.as_deref())?;
        }
        Commands::Keys { command } => {
            match command {
                KeyCommands::Generate { private, public, force } => {
                    keys::generate_key_pair_cmd(&private, &public, force)?;
                }
                KeyCommands::Info { key, type_ } => {
                    keys::show_key_info(&key, &type_)?;
                }
                KeyCommands::List { directory } => {
                    keys::list_keys(&directory)?;
                }
                KeyCommands::Export { private, public } => {
                    keys::export_public_key(&private, &public)?;
                }
                KeyCommands::Import { source, dest, type_ } => {
                    keys::import_key(&source, &dest, &type_)?;
                }
                KeyCommands::Backup { source, dest } => {
                    keys::backup_keys(&source, &dest)?;
                }
            }
        }
    }

    Ok(())
}
