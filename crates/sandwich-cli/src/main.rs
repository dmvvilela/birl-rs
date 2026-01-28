mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};
use sandwich_core::View;
use sandwich_storage::StorageService;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "sandwich-cli")]
#[command(about = "CLI tool for the Sandwich image composition app", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Use local filesystem instead of S3 (path to directory containing sandwich/)
    #[arg(short, long, global = true)]
    local: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compose a single image
    Compose {
        /// View to render (front, back, side, left, right)
        #[arg(long, default_value = "front")]
        view: String,

        /// Parameters: "category/sku,category/sku,..."
        #[arg(short, long, conflicts_with = "example")]
        params: Option<String>,

        /// Use a pre-made example
        #[arg(short, long)]
        example: Option<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Bypass cache and force regeneration
        #[arg(short, long)]
        bypass_cache: bool,
    },

    /// List available examples
    Examples,

    /// Show cache statistics
    Stats,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Create storage service (local or S3 based on --local flag)
    let storage = if let Some(local_path) = &cli.local {
        println!("Using local filesystem storage: {}", local_path.display());
        Arc::new(StorageService::new_local(local_path.clone(), 1000))
    } else {
        // Load AWS configuration
        let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let s3_client = aws_sdk_s3::Client::new(&aws_config);

        // Get bucket name from environment
        let bucket_name = std::env::var("AWS_BUCKET_NAME")
            .unwrap_or_else(|_| {
                eprintln!("Warning: AWS_BUCKET_NAME not set, using default");
                "sandwich-bucket".to_string()
            });

        println!("Using S3 storage: {}", bucket_name);
        #[allow(deprecated)]
        Arc::new(StorageService::new(s3_client, bucket_name, 1000))
    };

    // Execute command
    match cli.command {
        Commands::Compose {
            view,
            params,
            example,
            output,
            bypass_cache,
        } => {
            // Get parameters from example or direct input
            let params_string = if let Some(example_name) = example {
                let example = commands::examples::get_example(&example_name)
                    .ok_or_else(|| anyhow::anyhow!("Example '{}' not found", example_name))?;
                println!("Using example: {} - {}", example.name, example.description);
                example.params.to_string()
            } else if let Some(p) = params {
                p
            } else {
                anyhow::bail!("Either --params or --example must be provided");
            };

            // Parse view
            let view = parse_view(&view)?;

            // Execute compose command
            let options = commands::compose::ComposeOptions {
                view,
                params: params_string,
                output,
                bypass_cache,
            };

            commands::compose_command(storage, options).await?;
        }

        Commands::Examples => {
            commands::list_examples();
        }

        Commands::Stats => {
            let stats = storage.cache_stats().await;
            println!("Cache Statistics:");
            println!("  Memory entries: {}", stats.memory_entries);
            println!("  Memory capacity: {}", stats.memory_capacity);
        }
    }

    Ok(())
}

fn parse_view(view_str: &str) -> Result<View> {
    match view_str.to_lowercase().as_str() {
        "front" => Ok(View::Front),
        "back" => Ok(View::Back),
        "side" => Ok(View::Side),
        "left" => Ok(View::Left),
        "right" => Ok(View::Right),
        _ => anyhow::bail!("Invalid view: {}. Must be one of: front, back, side, left, right", view_str),
    }
}
