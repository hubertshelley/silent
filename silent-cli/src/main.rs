use anyhow::Result;
use clap::{Parser, Subcommand};
use fs_extra::dir::copy;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "silent-cli")]
#[command(about = "CLI tool for creating Silent web projects", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Silent web project
    New {
        /// Name of the project
        name: String,

        /// Path to create project (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, path } => {
            create_project(&name, path)?;
        }
    }

    Ok(())
}

fn create_project(name: &str, path: Option<PathBuf>) -> Result<()> {
    let target_dir = path.unwrap_or_else(|| PathBuf::from("."));
    let project_dir = target_dir.join(name);

    // Create progress bar
    let pb = ProgressBar::new(4);
    pb.set_style(
        ProgressStyle::default_bar().template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
        )?,
    );

    pb.set_message("Creating project structure...");
    fs::create_dir_all(&project_dir)?;
    pb.inc(1);

    pb.set_message("Copying template files...");
    copy_template(&project_dir)?;
    pb.inc(1);

    pb.set_message("Updating project configuration...");
    update_config(&project_dir, name)?;
    pb.inc(1);

    pb.set_message("Finalizing...");
    pb.inc(1);
    pb.finish_with_message("Project created successfully!");

    println!(
        "\n🎉 Successfully created project at: {}",
        project_dir.display()
    );
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  cargo run");

    Ok(())
}

fn copy_template(target: &PathBuf) -> Result<()> {
    let template_path = PathBuf::from("../silent-template");
    let options = fs_extra::dir::CopyOptions::new();
    copy(template_path, target, &options)?;
    Ok(())
}

fn update_config(project_dir: &Path, name: &str) -> Result<()> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml_path)?;
    let updated = content.replace("silent-template", name);
    fs::write(cargo_toml_path, updated)?;
    Ok(())
}
