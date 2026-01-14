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
use generator::{generate_bean_type_enums_xml, generate_enum_xml, XmlGenerator};
use parser::TsParser;
use ts_generator::TsCodeGenerator;
use tsconfig::TsConfig;
use type_mapper::TypeMapper;

mod table_registry;
use table_registry::TableRegistry;

mod table_mapping;
use table_mapping::TableMappingResolver;

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

    // Build class map for virtual field injection
    let all_classes = if !config.virtual_fields.is_empty() {
        use parser::inject_virtual_fields;
        use std::collections::HashMap;

        println!(
            "\n  Injecting {} virtual field config(s)...",
            config.virtual_fields.len()
        );

        let mut class_map: HashMap<String, parser::ClassInfo> = HashMap::new();
        for class in &all_classes {
            class_map.insert(class.name.clone(), class.clone());
        }

        inject_virtual_fields(&mut class_map, &config.virtual_fields)?;

        // Rebuild all_classes with injected fields
        let mut final_classes_with_virtual: Vec<parser::ClassInfo> = Vec::new();
        for class in &all_classes {
            if let Some(updated_class) = class_map.get(&class.name) {
                final_classes_with_virtual.push(updated_class.clone());
            }
        }
        final_classes_with_virtual
    } else {
        all_classes
    };

    // First pass: collect @LubanTable classes into registry for ref resolution
    // Use per-source module_name if set, otherwise use default config.output.module_name
    let default_module_name = &config.output.module_name;
    let mut table_registry = TableRegistry::new();
    for class in &all_classes {
        if class.luban_table.is_some() {
            let namespace = class
                .module_name
                .as_deref()
                .unwrap_or(default_module_name.as_str());
            table_registry.register(&class.name, namespace);
        }
    }
    if cli.verbose {
        println!(
            "  Registered {} tables in registry",
            all_classes
                .iter()
                .filter(|c| c.luban_table.is_some())
                .count()
        );
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

    // Generate XML - group by (output_path, module_name)
    println!("\n[4/4] Generating XML...");
    let table_mapping_resolver = TableMappingResolver::new(&config.table_mappings);
    let xml_generator = XmlGenerator::new(&type_mapper, &table_registry, &table_mapping_resolver);

    // Group classes by (output_path, module_name)
    let default_output = config.output.path.clone();
    let default_module = config.output.module_name.clone();
    let mut grouped: std::collections::HashMap<(PathBuf, String), Vec<_>> =
        std::collections::HashMap::new();
    for class in final_classes.iter() {
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

    // Generate enum XML if there are enums
    if !final_enums.is_empty() {
        // Group enums by (output_path, module_name)
        let mut enum_grouped: std::collections::HashMap<(PathBuf, String), Vec<_>> =
            std::collections::HashMap::new();

        // Default enum output path: use config.output.enum_path or derive from default_output
        let default_enum_output = config.output.enum_path.clone().unwrap_or_else(|| {
            default_output.with_file_name(format!(
                "{}_enums.xml",
                default_output
                    .file_stem()
                    .map(|s| s.to_string_lossy())
                    .unwrap_or_default()
            ))
        });

        for enum_info in final_enums.iter() {
            let out_path = enum_info
                .output_path
                .clone()
                .map(|p| {
                    // For per-source output_path, add _enums suffix
                    p.with_file_name(format!(
                        "{}_enums.xml",
                        p.file_stem()
                            .map(|s| s.to_string_lossy())
                            .unwrap_or_default()
                    ))
                })
                .unwrap_or_else(|| default_enum_output.clone());
            let module = enum_info
                .module_name
                .clone()
                .unwrap_or_else(|| default_module.clone());
            enum_grouped
                .entry((out_path, module))
                .or_default()
                .push(enum_info);
        }

        for ((out_path, module_name), enums) in &enum_grouped {
            let enums_owned: Vec<_> = enums.iter().map(|e| (*e).clone()).collect();
            let xml_output = generate_enum_xml(&enums_owned, module_name);

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
                println!("  Written {} enums to {:?}", enums.len(), resolved_path);
            }
        }
    }

    // Generate bean type enums XML if configured (grouped by parent)
    if let Some(bean_types_path) = &config.output.bean_types_path {
        // Collect beans with their extends (parent), aliases, and comments
        let beans_with_parents: Vec<(&str, String, Option<&str>, Option<&str>)> = final_classes
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
        let ts_generator = TsCodeGenerator::new(
            resolved_path.clone(),
            final_classes.clone(),
            tsconfig,
            config.output.module_name.clone(),
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
        final_classes.len(),
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
