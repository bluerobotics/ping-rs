use std::env;
use std::ffi::OsStr;
use std::fs::{read_dir, File};
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

use clap::Parser;
use anyhow::{Result, Context};
use tracing::{info, error, warn, debug};

#[path = "../../build/parser.rs"]
mod parser;
#[path = "../../build/binder.rs"]
mod binder;

#[derive(Parser)]
#[command(name = "ping-protocol-validator")]
#[command(about = "Validates ping-protocol submodule definitions")]
#[command(version)]
struct Cli {
    /// Path to the ping-protocol directory (defaults to current directory)
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Skip git submodule operations
    #[arg(long)]
    skip_git: bool,
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt::init();
    }

    info!("Starting ping-protocol validation...");

    match run_validation(&cli) {
        Ok(()) => {
            info!("✅ Validation completed successfully!");
            exit(0);
        }
        Err(e) => {
            error!("❌ Validation failed: {}", e);
            exit(1);
        }
    }
}

fn run_validation(cli: &Cli) -> Result<()> {
    let current_dir = env::current_dir()?;
    let protocol_dir = cli.path.as_ref().unwrap_or(&current_dir);

    info!("Validating ping-protocol at: {}", protocol_dir.display());

    // Check if we're in a ping-protocol directory
    let definitions_dir = protocol_dir.join("src/definitions");
    if !definitions_dir.exists() {
        return Err(anyhow::anyhow!(
            "Not a valid ping-protocol directory. Expected 'src/definitions' at {}",
            definitions_dir.display()
        ));
    }

    // Create a temporary output directory for validation
    let temp_dir = tempfile::tempdir()?;
    let out_dir = temp_dir.path();

    info!("Processing definition files...");
    let mut modules = vec![];
    let mut errors = vec![];

    for entry in read_dir(&definitions_dir)
        .with_context(|| format!("Could not read definitions directory: {}", definitions_dir.display()))?
    {
        let entry = entry?;
        let definition_file = entry.file_name();
        let module_name: String = definition_file.to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid filename: {:?}", definition_file))?
            .into();

        if module_name.contains("pinghf") {
            info!("Skipping pinghf file: {}", module_name);
            continue;
        }

        let module_name_without_ext = PathBuf::from(&module_name)
            .file_stem()
            .ok_or_else(|| anyhow::anyhow!("Invalid module name: {}", module_name))?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid module name encoding: {}", module_name))?
            .to_string();

        info!("Processing module: {}", module_name_without_ext);
        modules.push(module_name_without_ext.clone());

        let in_path = definitions_dir.join(&definition_file);
        let mut in_file = File::open(&in_path)
            .with_context(|| format!("Could not open definition file: {}", in_path.display()))?;

        let mut definition_rs = PathBuf::from(&module_name);
        definition_rs.set_extension("rs");
        let dest_path = out_dir.join(definition_rs);
        let mut out_file = File::create(&dest_path)
            .with_context(|| format!("Could not create output file: {}", dest_path.display()))?;

        // Validate by attempting to generate the module
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            parser::generate(&mut in_file, &mut out_file, &module_name_without_ext)
        })) {
            Ok(_) => {
                info!("✅ Successfully processed: {}", module_name_without_ext);
            }
            Err(e) => {
                let error_msg = if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Unknown panic".to_string()
                };
                error!("❌ Failed to process {}: {}", module_name_without_ext, error_msg);
                errors.push(format!("{}: {}", module_name_without_ext, error_msg));
            }
        }

    }

    if errors.is_empty() {
        info!("✅ Individual module generation validation completed successfully");
        info!("Skipping module binder generation (not essential for validation)");
    }

    if !errors.is_empty() {
        return Err(anyhow::anyhow!(
            "Validation failed with {} error(s):\n{}",
            errors.len(),
            errors.join("\n")
        ));
    }

    info!("All definition files processed successfully!");
    Ok(())
}
