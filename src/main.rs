use anyhow::{Context, Result};
use clap::Parser;
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::Instant;

mod config;
mod tsconfig;
mod parser;
mod type_mapper;
mod base_class;
mod generator;
mod cache;
mod scanner;

use config::{Config, SourceConfig};
use tsconfig::TsConfig;
use parser::TsParser;
use type_mapper::TypeMapper;
use base_class::BaseClassResolver;
use generator::XmlGenerator;
use cache::Cache;

#[derive(Parser)]
#[command(name = "luban-gen")]
#[command(about = "High-performance TypeScript to Luban XML Schema generator")]
#[command(version)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "luban.config.toml")]
    config: PathBuf,

    /// Force regenerate all beans (ignore cache)
    #[arg(short, long)]
    force: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let start = Instant::now();

    println!("Luban Schema Generator v{}", env!("CARGO_PKG_VERSION"));
    println!("{}", "=".repeat(50));

    // Load configuration
    let config = Config::load(&cli.config)
        .with_context(|| format!("Failed to load config from {:?}", cli.config))?;

    let project_root = cli.config.parent().unwrap_or_else(|| std::path::Path::new("."));

    // Load tsconfig for path resolution
    let tsconfig_path = project_root.join(&config.project.tsconfig);
    let tsconfig = TsConfig::load(&tsconfig_path)
        .with_context(|| format!("Failed to load tsconfig from {:?}", tsconfig_path))?;

    let _path_resolver = tsconfig::PathResolver::new(&tsconfig, project_root);

    // Initialize components
    let type_mapper = TypeMapper::new(&config.type_mappings);
    let base_resolver = BaseClassResolver::new(
        &config.base_class_mappings,
        &config.defaults.base_class,
    );

    // Load cache
    let cache_path = project_root.join(&config.output.cache_file);
    let mut cache = if cli.force {
        println!("[Force mode] Ignoring cache, regenerating all beans...");
        Cache::new()
    } else {
        Cache::load(&cache_path).unwrap_or_default()
    };

    // Collect source directories
    let mut source_dirs = Vec::new();
    for source in &config.sources {
        match source {
            SourceConfig::Directory { path } => {
                source_dirs.push(project_root.join(path));
            }
            SourceConfig::Registration { path } => {
                // TODO: Parse registration file
                println!("  Registration mode not yet implemented: {:?}", path);
            }
        }
    }

    // Scan for TypeScript files
    println!("\n[1/4] Scanning directories...");
    let ts_files = scanner::scan_directories(&source_dirs)?;
    println!("  Found {} TypeScript files", ts_files.len());

    // Parse files in parallel
    println!("\n[2/4] Parsing TypeScript files...");

    let all_classes: Vec<_> = ts_files
        .par_iter()
        .filter_map(|path| {
            // Create parser per-thread since SourceMap isn't Sync
            let ts_parser = TsParser::new();
            match ts_parser.parse_file(path) {
                Ok(classes) => Some(classes),
                Err(e) => {
                    eprintln!("  Warning: Failed to parse {:?}: {}", path, e);
                    None
                }
            }
        })
        .flatten()
        .collect();

    println!("  Extracted {} classes/interfaces", all_classes.len());

    // Filter by cache
    println!("\n[3/4] Checking cache...");
    let mut unchanged = 0;
    let mut updated = 0;

    let final_classes: Vec<_> = all_classes
        .into_iter()
        .inspect(|class| {
            if cache.is_valid(&class.name, &class.file_hash) {
                unchanged += 1;
                if cli.verbose {
                    println!("  [cached] {}", class.name);
                }
            } else {
                updated += 1;
                if cli.verbose {
                    println!("  [update] {}", class.name);
                }
                cache.set_entry(&class.name, &class.source_file, &class.file_hash);
            }
        })
        .collect();

    println!("  Cached: {}, Updated: {}", unchanged, updated);

    // Generate XML
    println!("\n[4/4] Generating XML...");
    let xml_generator = XmlGenerator::new(&base_resolver, &type_mapper);
    let xml_output = xml_generator.generate(&final_classes);

    // Write output only if changed
    let output_path = project_root.join(&config.output.path);
    let should_write = if output_path.exists() {
        let existing = std::fs::read_to_string(&output_path)?;
        existing != xml_output
    } else {
        true
    };

    if should_write {
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&output_path, &xml_output)?;
        println!("  Written to {:?}", output_path);
    } else {
        println!("  No changes, skipping write");
    }

    // Save cache
    cache.save(&cache_path)?;

    let elapsed = start.elapsed();
    println!("\n{}", "=".repeat(50));
    println!("Done! Generated {} beans in {:?}", final_classes.len(), elapsed);

    Ok(())
}
