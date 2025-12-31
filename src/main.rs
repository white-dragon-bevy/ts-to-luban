use anyhow::Result;

mod config;
mod parser;
mod generator;
mod cache;

fn main() -> Result<()> {
    println!("Luban Schema Generator v0.1.0");
    Ok(())
}
