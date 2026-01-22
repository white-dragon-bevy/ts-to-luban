#![allow(dead_code)]

use anyhow::{Context, Result};
use clap::Parser;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

mod cache;
mod config;
mod generator;
mod parser;
mod scanner;
mod ts_generator;
mod tsconfig;
mod type_mapper;

use cache::Cache;
use config::{Config, SourceConfig};
use generator::{generate_bean_type_enums_xml, XmlGenerator};
use parser::TsParser;
use ts_generator::TsCodeGenerator;
use tsconfig::TsConfig;
use type_mapper::TypeMapper;

mod table_registry;
use table_registry::TableRegistry;

mod table_mapping;
// TableMappingResolver is deprecated - tables are now configured in [tables] section

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

    /// Watch mode: monitor source files for changes and regenerate
    #[arg(short, long)]
    watch: bool,
}

/// Run the generation process once
fn run_generation(
    cli: &Cli,
    config: &Config,
    project_root: &Path,
    tsconfig: &TsConfig,
) -> Result<()> {
    let start = Instant::now();

    println!("Luban Schema Generator v{}", env!("CARGO_PKG_VERSION"));
    println!("{}", "=".repeat(50));

    // Initialize components
    let type_mapper = TypeMapper::new(&config.type_mappings);

    // Load cache
    let cache_path = project_root.join(&config.output.cache_file);
    let mut cache = if cli.force {
        println!("[Force mode] Ignoring cache, regenerating all beans...");
        Cache::new()
    } else {
        Cache::load(&cache_path).unwrap_or_default()
    };

    // Collect source files and directories with their output paths and module names
    let mut source_dirs: Vec<(
        PathBuf,
        scanner::ScanConfig,
        Option<PathBuf>,
        Option<String>,
    )> = Vec::new();
    let mut single_files: Vec<(PathBuf, Option<PathBuf>, Option<String>)> = Vec::new();

    for source in &config.sources {
        match source {
            SourceConfig::Directory {
                path,
                scan_options,
                output_path,
                module_name,
            } => {
                let scan_config = scanner::ScanConfig::from(scan_options);
                let resolved = if path.is_absolute() {
                    path.clone()
                } else {
                    project_root.join(path)
                };
                if !resolved.exists() {
                    anyhow::bail!("Source directory not found: {:?}", resolved);
                }
                if !resolved.is_dir() {
                    anyhow::bail!("Source path is not a directory: {:?}", resolved);
                }
                source_dirs.push((
                    resolved,
                    scan_config,
                    output_path.clone(),
                    module_name.clone(),
                ));
            }
            SourceConfig::File {
                path,
                output_path,
                module_name,
            } => {
                let resolved = if path.is_absolute() {
                    path.clone()
                } else {
                    project_root.join(path)
                };
                if !resolved.exists() {
                    anyhow::bail!("Source file not found: {:?}", resolved);
                }
                if !resolved.is_file() {
                    anyhow::bail!("Source path is not a file: {:?}", resolved);
                }
                single_files.push((resolved, output_path.clone(), module_name.clone()));
            }
            SourceConfig::Files {
                paths,
                output_path,
                module_name,
            } => {
                for path in paths {
                    let resolved = if path.is_absolute() {
                        path.clone()
                    } else {
                        project_root.join(path)
                    };
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
            SourceConfig::Glob {
                pattern,
                output_path,
                module_name,
            } => {
                // Resolve pattern relative to project_root if not already absolute
                let resolved_pattern = if Path::new(pattern).is_absolute() {
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

    let parse_results: Vec<_> = ts_files
        .par_iter()
        .filter_map(|(path, output_path, module_name)| {
            // Create parser per-thread since SourceMap isn't Sync
            let ts_parser = TsParser::new();
            let classes = match ts_parser.parse_file(path) {
                Ok(mut classes) => {
                    // Set output_path and module_name for all classes from this file
                    for class in &mut classes {
                        class.output_path = output_path.clone();
                        class.module_name = module_name.clone();
                    }
                    classes
                }
                Err(e) => {
                    eprintln!("  Warning: Failed to parse classes from {:?}: {}", path, e);
                    vec![]
                }
            };
            let enums = match ts_parser.parse_enums(path) {
                Ok(mut enums) => {
                    for e in &mut enums {
                        e.output_path = output_path.clone();
                        e.module_name = module_name.clone();
                    }
                    enums
                }
                Err(e) => {
                    eprintln!("  Warning: Failed to parse enums from {:?}: {}", path, e);
                    vec![]
                }
            };
            Some((classes, enums))
        })
        .collect();

    let all_classes: Vec<_> = parse_results.iter().flat_map(|(c, _)| c.clone()).collect();
    let all_enums: Vec<_> = parse_results.iter().flat_map(|(_, e)| e.clone()).collect();

    println!(
        "  Extracted {} classes/interfaces, {} enums",
        all_classes.len(),
        all_enums.len()
    );

    // Build table registry from [tables] config
    let mut table_registry = TableRegistry::from_config(&config.tables);
    
    // Set index types based on parsed class information
    table_registry.set_index_types(&all_classes, &type_mapper);
    
    if cli.verbose {
        println!(
            "  Registered {} tables from [tables] config",
            config.tables.len()
        );
    }

    // Validate that all configured tables have corresponding beans
    if !config.tables.is_empty() {
        let existing_beans: std::collections::HashSet<String> = all_classes
            .iter()
            .map(|class| {
                if let Some(module) = &class.module_name {
                    format!("{}.{}", module, class.name)
                } else {
                    class.name.clone()
                }
            })
            .collect();

        let missing_beans = table_registry.validate_beans_exist(&existing_beans);
        if !missing_beans.is_empty() {
            eprintln!("\nError: The following tables are configured but their beans do not exist:");
            for bean in &missing_beans {
                eprintln!("  - {}", bean);
            }
            eprintln!("\nPlease check your [tables] configuration and ensure the corresponding TypeScript classes/interfaces exist.");
            std::process::exit(1);
        }
    }

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

    // Filter enums by cache
    let final_enums: Vec<_> = all_enums
        .into_iter()
        .inspect(|enum_info| {
            if cache.is_valid(&enum_info.name, &enum_info.file_hash) {
                unchanged += 1;
                if cli.verbose {
                    println!("  [cached enum] {}", enum_info.name);
                }
            } else {
                updated += 1;
                if cli.verbose {
                    println!("  [update enum] {}", enum_info.name);
                }
                cache.set_entry(
                    &enum_info.name,
                    &enum_info.source_file,
                    &enum_info.file_hash,
                );
            }
        })
        .collect();

    println!("  Cached: {}, Updated: {}", unchanged, updated);

    // No longer need table_mapping_resolver - tables are configured in [tables] section
    let final_classes_with_table_names: Vec<_> = final_classes;

    // Generate XML - group by (output_path, module_name)
    println!("\n[4/4] Generating XML...");

    // Build type-to-module mapping including enums
    let mut type_to_module: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for enum_info in &final_enums {
        if let Some(module) = &enum_info.module_name {
            type_to_module.insert(enum_info.name.clone(), module.clone());
        }
    }

    // Build file-to-module mapping for cross-module type resolution
    // This maps source file paths to their module names
    let mut file_to_module: std::collections::HashMap<std::path::PathBuf, String> = std::collections::HashMap::new();
    for class in &final_classes_with_table_names {
        if let Some(module) = &class.module_name {
            // Canonicalize the source file path for consistent matching
            let source_path = std::path::PathBuf::from(&class.source_file);
            if let Ok(canonical) = source_path.canonicalize() {
                file_to_module.insert(canonical, module.clone());
            } else {
                // If canonicalize fails, use the original path
                file_to_module.insert(source_path, module.clone());
            }
        }
    }
    // Also add enums to file_to_module
    for enum_info in &final_enums {
        if let Some(module) = &enum_info.module_name {
            let source_path = std::path::PathBuf::from(&enum_info.source_file);
            if let Ok(canonical) = source_path.canonicalize() {
                file_to_module.insert(canonical, module.clone());
            } else {
                file_to_module.insert(source_path, module.clone());
            }
        }
    }

    let xml_generator = XmlGenerator::with_type_and_file_mapping(&type_mapper, &table_registry, type_to_module, file_to_module);

    // Group classes by (output_path, module_name)
    let default_output = config.output.path.clone();
    let default_module = config.output.module_name.clone();
    let mut grouped: std::collections::HashMap<(PathBuf, String), Vec<_>> =
        std::collections::HashMap::new();
    for class in final_classes_with_table_names.iter() {
        let out_path = class
            .output_path
            .clone()
            .unwrap_or_else(|| default_output.clone());
        let module = class
            .module_name
            .clone()
            .unwrap_or_else(|| default_module.clone());
        grouped.entry((out_path, module)).or_default().push(class);
    }

    // Group enums by (output_path, module_name) - same grouping as classes
    let mut enum_grouped: std::collections::HashMap<(PathBuf, String), Vec<_>> =
        std::collections::HashMap::new();
    for enum_info in final_enums.iter() {
        let out_path = enum_info
            .output_path
            .clone()
            .unwrap_or_else(|| default_output.clone());
        let module = enum_info
            .module_name
            .clone()
            .unwrap_or_else(|| default_module.clone());
        enum_grouped
            .entry((out_path, module))
            .or_default()
            .push(enum_info);
    }

    // Collect all unique (output_path, module_name) keys from both classes and enums
    let mut all_keys: std::collections::HashSet<(PathBuf, String)> = std::collections::HashSet::new();
    for key in grouped.keys() {
        all_keys.insert(key.clone());
    }
    for key in enum_grouped.keys() {
        all_keys.insert(key.clone());
    }

    // Generate and write each group (classes + enums merged into same file)
    let mut files_written = 0;
    for (out_path, module_name) in &all_keys {
        let classes = grouped.get(&(out_path.clone(), module_name.clone()));
        let enums = enum_grouped.get(&(out_path.clone(), module_name.clone()));

        let classes_owned: Vec<_> = classes
            .map(|c| c.iter().map(|c| (*c).clone()).collect())
            .unwrap_or_default();
        let enums_owned: Vec<_> = enums
            .map(|e| e.iter().map(|e| (*e).clone()).collect())
            .unwrap_or_default();

        let xml_output = xml_generator.generate_with_all_classes_and_enums(
            &classes_owned,
            &enums_owned,
            module_name,
            &final_classes_with_table_names,
        );

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
            println!(
                "  Written {} beans, {} enums to {:?}",
                classes_owned.len(),
                enums_owned.len(),
                resolved_path
            );
            files_written += 1;
        } else {
            println!("  No changes for {:?}", resolved_path);
        }
    }

    if files_written == 0 {
        println!("  No changes, skipping all writes");
    }

    // Generate bean type enums XML if configured (grouped by parent)
    if let Some(bean_types_path) = &config.output.bean_types_path {
        // Collect beans with their extends (parent), aliases, and comments
        let beans_with_parents: Vec<(&str, String, Option<&str>, Option<&str>)> = final_classes_with_table_names
            .iter()
            .map(|c| {
                (
                    c.name.as_str(),
                    c.extends.clone().unwrap_or_default(),
                    c.alias.as_deref(),
                    c.comment.as_deref(),
                )
            })
            .collect();
        let beans_refs: Vec<(&str, &str, Option<&str>, Option<&str>)> = beans_with_parents
            .iter()
            .map(|(name, parent, alias, comment)| (*name, parent.as_str(), *alias, *comment))
            .collect();

        let xml_output = generate_bean_type_enums_xml(&beans_refs, &default_module);

        let resolved_path = project_root.join(bean_types_path);
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
            println!("  Written bean type enums to {:?}", resolved_path);
        }
    }

    // Generate TypeScript table code if configured
    if let Some(table_output_path) = &config.output.table_output_path {
        println!("\n[5/5] Generating TypeScript table code...");

        let resolved_path = project_root.join(table_output_path);

        // For TypeScript generation, determine the true project root
        // (where assets/ folder is located)
        // Check if we're in a subdirectory by looking for parent indicators
        let ts_project_root = if project_root
            .parent()
            .map(|p| p.join("Cargo.toml").exists() || p.join("assets").exists())
            .unwrap_or(false)
        {
            // Parent has Cargo.toml or assets/, use parent as project root
            project_root
                .parent()
                .unwrap_or(project_root)
                .to_path_buf()
        } else {
            // Use current project_root
            project_root.to_path_buf()
        };

        let ts_generator = TsCodeGenerator::new(
            resolved_path.clone(),
            ts_project_root,
            final_classes_with_table_names.clone(),
            tsconfig,
            config.output.module_name.clone(),
            &table_registry,
        );

        ts_generator.generate()?;
        println!("  Written TypeScript tables to {:?}", resolved_path);
    }

    // Save cache
    cache.save(&cache_path)?;

    let elapsed = start.elapsed();
    println!("\n{}", "=".repeat(50));
    println!(
        "Done! Generated {} beans, {} enums to {} file(s) in {:?}",
        final_classes_with_table_names.len(),
        final_enums.len(),
        grouped.len(),
        elapsed
    );

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration with ref_configs merging
    let config = Config::load_with_refs(&cli.config)
        .with_context(|| format!("Failed to load config from {:?}", cli.config))?;

    let project_root = cli
        .config
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    // Load tsconfig (for path resolution support)
    let tsconfig_path = project_root.join(&config.project.tsconfig);
    let tsconfig = TsConfig::load(&tsconfig_path)
        .with_context(|| format!("Failed to load tsconfig from {:?}", tsconfig_path))?;

    // Run generation once if not in watch mode
    if !cli.watch {
        run_generation(&cli, &config, project_root, &tsconfig)?;
        return Ok(());
    }

    // Watch mode: monitor source files for changes and regenerate
    println!("Watch mode enabled. Monitoring for changes...");
    println!("Press Ctrl+C to stop.\n");

    // Collect paths to watch
    let mut watch_paths: Vec<PathBuf> = Vec::new();
    for source in &config.sources {
        match source {
            SourceConfig::Directory { path, .. } => {
                let resolved = if path.is_absolute() {
                    path.clone()
                } else {
                    project_root.join(path)
                };
                watch_paths.push(resolved);
            }
            SourceConfig::File { path, .. } => {
                let resolved = if path.is_absolute() {
                    path.clone()
                } else {
                    project_root.join(path)
                };
                // Watch parent directory for single files
                if let Some(parent) = resolved.parent() {
                    if !watch_paths.contains(&parent.to_path_buf()) {
                        watch_paths.push(parent.to_path_buf());
                    }
                }
            }
            SourceConfig::Files { paths, .. } => {
                for path in paths {
                    let resolved = if path.is_absolute() {
                        path.clone()
                    } else {
                        project_root.join(path)
                    };
                    // Watch parent directory for single files
                    if let Some(parent) = resolved.parent() {
                        if !watch_paths.contains(&parent.to_path_buf()) {
                            watch_paths.push(parent.to_path_buf());
                        }
                    }
                }
            }
            SourceConfig::Glob { pattern, .. } => {
                // Expand glob to get initial files
                let resolved_pattern = if Path::new(pattern).is_absolute() {
                    pattern.clone()
                } else {
                    project_root.join(pattern).to_string_lossy().to_string()
                };
                if let Ok(files) = scanner::expand_glob(&resolved_pattern) {
                    for file in files {
                        if let Some(parent) = file.parent() {
                            if !watch_paths.contains(&parent.to_path_buf()) {
                                watch_paths.push(parent.to_path_buf());
                            }
                        }
                    }
                }
            }
            SourceConfig::Registration { .. } => {
                // Registration mode not implemented, skip
            }
        }
    }

    // Deduplicate watch paths
    watch_paths.sort();
    watch_paths.dedup();
    watch_paths.retain(|p| p.exists());

    if watch_paths.is_empty() {
        println!("No valid paths to watch. Exiting.");
        return Ok(());
    }

    println!("Watching paths:");
    for path in &watch_paths {
        println!("  {}", path.display());
    }
    println!();

    // Create channel for file system events
    let (tx, rx) = channel();

    // Create watcher
    let mut watcher: RecommendedWatcher = Watcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if let Err(e) = tx.send(event) {
                    eprintln!("Failed to send file event: {}", e);
                }
            }
        },
        notify::Config::default(),
    )?;

    // Watch all paths
    for path in &watch_paths {
        watcher.watch(path, RecursiveMode::Recursive)?;
    }

    // Debounce delay (in milliseconds)
    const DEBOUNCE_MS: u64 = 300;

    // Event loop with debouncing
    let mut last_change_time = Instant::now();
    let mut pending_generation = false;

    loop {
        // Check for file events
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) {
            // Filter for relevant events (create, modify, remove, rename)
            if matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) | EventKind::Any
            ) {
                // Only care about TypeScript files
                if event.paths.iter().any(|p| {
                    p.extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == "ts" || ext == "mts" || ext == "cts")
                        .unwrap_or(false)
                }) {
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_change_time).as_millis() as u64;

                    if elapsed >= DEBOUNCE_MS {
                        // Debounce period passed, regenerate
                        pending_generation = true;
                        last_change_time = now;
                    } else {
                        // Reset the timer (debounce)
                        last_change_time = now;
                    }
                }
            }
        }

        // Perform generation if pending and debounce period passed
        if pending_generation {
            let elapsed = last_change_time.elapsed().as_millis() as u64;
            if elapsed >= DEBOUNCE_MS {
                println!("\nChanges detected, regenerating...");
                if let Err(e) = run_generation(&cli, &config, project_root, &tsconfig) {
                    eprintln!("Error during generation: {}", e);
                }
                println!("\nWatching for changes (press Ctrl+C to stop)...\n");
                pending_generation = false;
            }
        }
    }
}
