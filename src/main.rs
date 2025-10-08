use clap::{Parser, Subcommand};
use paxbuild::{build, verify, sign, info, extract};

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
    }

    Ok(())
}
