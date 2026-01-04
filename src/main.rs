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

    // Load configuration with ref_configs merging
    let config = Config::load_with_refs(&cli.config)
        .with_context(|| format!("Failed to load config from {:?}", cli.config))?;

    let project_root = cli.config.parent().unwrap_or_else(|| std::path::Path::new("."));

    // Load tsconfig (for future path resolution support)
    let tsconfig_path = project_root.join(&config.project.tsconfig);
    let _tsconfig = TsConfig::load(&tsconfig_path)
        .with_context(|| format!("Failed to load tsconfig from {:?}", tsconfig_path))?;

    // Initialize components
    let type_mapper = TypeMapper::new(&config.type_mappings);
    let base_resolver = BaseClassResolver::new(&config.defaults.base_class, &config.parent_mappings);

    // Load cache
    let cache_path = project_root.join(&config.output.cache_file);
    let mut cache = if cli.force {
        println!("[Force mode] Ignoring cache, regenerating all beans...");
        Cache::new()
    } else {
        Cache::load(&cache_path).unwrap_or_default()
    };

    // Collect source files and directories with their output paths and module names
    // Note: paths from ref_configs are already resolved as absolute paths
    let mut source_dirs: Vec<(PathBuf, scanner::ScanConfig, Option<PathBuf>, Option<String>)> = Vec::new();
    let mut single_files: Vec<(PathBuf, Option<PathBuf>, Option<String>)> = Vec::new();
    for source in &config.sources {
        match source {
            SourceConfig::Directory { path, scan_options, output_path, module_name } => {
                let scan_config = scanner::ScanConfig::from(scan_options);
                let resolved = if path.is_absolute() { path.clone() } else { project_root.join(path) };
                if !resolved.exists() {
                    anyhow::bail!("Source directory not found: {:?}", resolved);
                }
                if !resolved.is_dir() {
                    anyhow::bail!("Source path is not a directory: {:?}", resolved);
                }
                source_dirs.push((resolved, scan_config, output_path.clone(), module_name.clone()));
            }
            SourceConfig::File { path, output_path, module_name } => {
                let resolved = if path.is_absolute() { path.clone() } else { project_root.join(path) };
                if !resolved.exists() {
                    anyhow::bail!("Source file not found: {:?}", resolved);
                }
                if !resolved.is_file() {
                    anyhow::bail!("Source path is not a file: {:?}", resolved);
                }
                single_files.push((resolved, output_path.clone(), module_name.clone()));
            }
            SourceConfig::Files { paths, output_path, module_name } => {
                for path in paths {
                    let resolved = if path.is_absolute() { path.clone() } else { project_root.join(path) };
                    if !resolved.exists() {
                        anyhow::bail!("Source file not found: {:?}", resolved);
                    }
                    if !resolved.is_file() {
                        anyhow::bail!("Source path is not a file: {:?}", resolved);
                    }
                    single_files.push((resolved, output_path.clone(), module_name.clone()));
                }
            }
            SourceConfig::Registration { path } => {
                // TODO: Parse registration file
                println!("  Registration mode not yet implemented: {:?}", path);
            }
            SourceConfig::Glob { pattern, output_path, module_name } => {
                // Resolve pattern relative to project_root if not already absolute
                let resolved_pattern = if std::path::Path::new(pattern).is_absolute() {
                    pattern.clone()
                } else {
                    project_root.join(pattern).to_string_lossy().to_string()
                };
                let files = scanner::expand_glob(&resolved_pattern)?;
                for file in files {
                    single_files.push((file, output_path.clone(), module_name.clone()));
                }
            }
        }
    }

    // Scan for TypeScript files and track their output paths and module names
    println!("\n[1/4] Scanning sources...");
    let mut ts_files: Vec<(PathBuf, Option<PathBuf>, Option<String>)> = Vec::new();

    for (dir, scan_config, output_path, module_name) in &source_dirs {
        let files = scanner::scan_directory_with_options(dir, scan_config)?;
        for file in files {
            ts_files.push((file, output_path.clone(), module_name.clone()));
        }
    }
    ts_files.extend(single_files);
    println!("  Found {} TypeScript files", ts_files.len());

    // Parse files in parallel, setting output_path and module_name for each class
    println!("\n[2/4] Parsing TypeScript files...");

    let all_classes: Vec<_> = ts_files
        .par_iter()
        .filter_map(|(path, output_path, module_name)| {
            // Create parser per-thread since SourceMap isn't Sync
            let ts_parser = TsParser::new();
            match ts_parser.parse_file(path) {
                Ok(mut classes) => {
                    // Set output_path and module_name for all classes from this file
                    for class in &mut classes {
                        class.output_path = output_path.clone();
                        class.module_name = module_name.clone();
                    }
                    Some(classes)
                }
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

    // Generate XML - group by (output_path, module_name)
    println!("\n[4/4] Generating XML...");
    let xml_generator = XmlGenerator::new(&base_resolver, &type_mapper);

    // Group classes by (output_path, module_name)
    let default_output = config.output.path.clone();
    let default_module = config.output.module_name.clone();
    let mut grouped: std::collections::HashMap<(PathBuf, String), Vec<_>> = std::collections::HashMap::new();
    for class in final_classes.iter() {
        let out_path = class.output_path.clone().unwrap_or_else(|| default_output.clone());
        let module = class.module_name.clone().unwrap_or_else(|| default_module.clone());
        grouped.entry((out_path, module)).or_default().push(class);
    }

    // Generate and write each group
    let mut files_written = 0;
    for ((out_path, module_name), classes) in &grouped {
        let classes_owned: Vec<_> = classes.iter().map(|c| (*c).clone()).collect();
        let xml_output = xml_generator.generate(&classes_owned, module_name);

        let resolved_path = project_root.join(out_path);
        let should_write = if resolved_path.exists() {
            let existing = std::fs::read_to_string(&resolved_path)?;
            existing != xml_output
        } else {
            true
        };

        if should_write {
            if let Some(parent) = resolved_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&resolved_path, &xml_output)?;
            println!("  Written {} beans to {:?}", classes.len(), resolved_path);
            files_written += 1;
        } else {
            println!("  No changes for {:?}", resolved_path);
        }
    }

    if files_written == 0 {
        println!("  No changes, skipping all writes");
    }

    // Save cache
    cache.save(&cache_path)?;

    let elapsed = start.elapsed();
    println!("\n{}", "=".repeat(50));
    println!("Done! Generated {} beans to {} file(s) in {:?}", final_classes.len(), grouped.len(), elapsed);

    Ok(())
}
