use anyhow::Result;
use sandwich_core::{compose_layers, generate_cache_key, parse_params, LayerNormalizer, View};
use sandwich_storage::StorageService;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

pub struct BenchmarkResults {
    pub test_name: String,
    pub iterations: usize,
    pub total_time: Duration,
    pub avg_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
}

impl BenchmarkResults {
    fn new(test_name: String, times: Vec<Duration>) -> Self {
        let iterations = times.len();
        let total_time: Duration = times.iter().sum();
        let avg_time = total_time / iterations as u32;
        let min_time = *times.iter().min().unwrap();
        let max_time = *times.iter().max().unwrap();

        Self {
            test_name,
            iterations,
            total_time,
            avg_time,
            min_time,
            max_time,
        }
    }

    fn print(&self) {
        println!("\n{}", "=".repeat(60));
        println!("Test: {}", self.test_name);
        println!("{}", "=".repeat(60));
        println!("Iterations:  {}", self.iterations);
        println!("Total time:  {:?}", self.total_time);
        println!("Average:     {:?}", self.avg_time);
        println!("Min:         {:?}", self.min_time);
        println!("Max:         {:?}", self.max_time);
    }

    fn to_markdown(&self) -> String {
        format!(
            "| {} | {} | {:.2}ms | {:.2}ms | {:.2}ms | {:.2}ms |",
            self.test_name,
            self.iterations,
            self.total_time.as_secs_f64() * 1000.0,
            self.avg_time.as_secs_f64() * 1000.0,
            self.min_time.as_secs_f64() * 1000.0,
            self.max_time.as_secs_f64() * 1000.0,
        )
    }
}

async fn bench_composition(
    storage: &StorageService,
    view: View,
    params: &str,
    iterations: usize,
) -> Result<Vec<Duration>> {
    let mut times = Vec::new();
    let mut fetch_times = Vec::new();
    let mut compose_times = Vec::new();

    for i in 0..iterations {
        let start = Instant::now();

        // Parse and normalize
        let params = parse_params(params);
        let normalizer = LayerNormalizer::new(view, &params);
        let normalized_params = normalizer.normalize_all(&params);

        // Fetch base plate and layers
        let fetch_start = Instant::now();
        let base_image_data = storage.fetch_base_plate(view).await?;
        let layers_result = storage.fetch_layers(&normalized_params, view).await?;
        let layers: Vec<_> = layers_result.into_iter().flatten().collect();
        fetch_times.push(fetch_start.elapsed());

        // Compose
        let compose_start = Instant::now();
        let _composite_data = compose_layers(&base_image_data, layers)?;
        compose_times.push(compose_start.elapsed());

        times.push(start.elapsed());
    }

    if iterations > 0 {
        let avg_fetch: Duration = fetch_times.iter().sum::<Duration>() / iterations as u32;
        let avg_compose: Duration = compose_times.iter().sum::<Duration>() / iterations as u32;
        println!("  → Avg I/O time: {:?}", avg_fetch);
        println!("  → Avg composition time: {:?}", avg_compose);
    }

    Ok(times)
}

async fn bench_with_cache(
    storage: &StorageService,
    view: View,
    params: &str,
    iterations: usize,
) -> Result<Vec<Duration>> {
    let mut times = Vec::new();

    // First composition to warm up cache
    let params_parsed = parse_params(params);
    let normalizer = LayerNormalizer::new(view, &params_parsed);
    let normalized_params = normalizer.normalize_all(&params_parsed);

    let base_image_data = storage.fetch_base_plate(view).await?;
    let layers_result = storage.fetch_layers(&normalized_params, view).await?;
    let layers: Vec<_> = layers_result.into_iter().flatten().collect();
    let composite_data = compose_layers(&base_image_data, layers)?;

    // Save to cache
    let cache_key = generate_cache_key(&normalized_params, view, view.plate_value());
    storage.save_composite(&cache_key, composite_data).await?;

    // Now benchmark cache retrieval
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = storage.get_cached_composite(&cache_key).await?;
        times.push(start.elapsed());
    }

    Ok(times)
}

pub async fn run_benchmarks(storage: Arc<StorageService>, output_file: Option<String>) -> Result<()> {
    println!("\n🚀 Running Sandwich Rust Benchmarks\n");

    let mut all_results = Vec::new();

    // Test 1: Basic composition (single item)
    info!("Running: Basic composition (single hoodie)");
    let times = bench_composition(&storage, View::Front, "hoodies/baerskin4-black", 10).await?;
    let result = BenchmarkResults::new("Basic (1 item)".to_string(), times);
    result.print();
    all_results.push(result);

    // Test 2: Full outfit (3 items)
    info!("Running: Full outfit composition");
    let times = bench_composition(
        &storage,
        View::Front,
        "hoodies/baerskin4-black,pants/cargo-darkgreen,hats/beanie-black",
        10,
    )
    .await?;
    let result = BenchmarkResults::new("Full outfit (3 items)".to_string(), times);
    result.print();
    all_results.push(result);

    // Test 3: Complex outfit (5 items)
    info!("Running: Complex outfit composition");
    let times = bench_composition(
        &storage,
        View::Front,
        "hoodies/baerskin4-black,pants/cargo-black,hats/beanie-black,gloves/baerskinleatherlinedgloves-black,jackets/softshell-grey",
        10,
    )
    .await?;
    let result = BenchmarkResults::new("Complex outfit (5 items)".to_string(), times);
    result.print();
    all_results.push(result);

    // Test 4: Different views
    info!("Running: Back view composition");
    let times = bench_composition(&storage, View::Back, "hoodies/baerskin4-black,pants/cargo-darkgreen", 10).await?;
    let result = BenchmarkResults::new("Back view (2 items)".to_string(), times);
    result.print();
    all_results.push(result);

    // Test 5: Cache performance
    info!("Running: Cache retrieval performance");
    let times = bench_with_cache(&storage, View::Front, "hoodies/baerskin4-black", 100).await?;
    let result = BenchmarkResults::new("Cache hit".to_string(), times);
    result.print();
    all_results.push(result);

    // Generate summary
    println!("\n{}", "=".repeat(60));
    println!("BENCHMARK SUMMARY");
    println!("{}", "=".repeat(60));
    println!("\n| Test | Iterations | Total | Avg | Min | Max |");
    println!("|------|------------|-------|-----|-----|-----|");
    for result in &all_results {
        println!("{}", result.to_markdown());
    }

    // Save to file if requested
    if let Some(output_path) = output_file {
        let mut output = String::new();
        output.push_str("# Sandwich Rust - Performance Benchmarks\n\n");
        output.push_str(&format!("**Date:** {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));

        output.push_str("## Results\n\n");
        output.push_str("| Test | Iterations | Total (ms) | Avg (ms) | Min (ms) | Max (ms) |\n");
        output.push_str("|------|------------|------------|----------|----------|----------|\n");
        for result in &all_results {
            output.push_str(&result.to_markdown());
            output.push_str("\n");
        }

        output.push_str("\n## System Information\n\n");
        output.push_str(&format!("- **OS:** {}\n", std::env::consts::OS));
        output.push_str(&format!("- **Architecture:** {}\n", std::env::consts::ARCH));
        output.push_str("- **Rust Version:** See Cargo.toml\n");
        output.push_str("- **Build:** Development (unoptimized)\n");

        output.push_str("\n## Notes\n\n");
        output.push_str("- All tests run with local filesystem storage\n");
        output.push_str("- Images loaded from local resources directory\n");
        output.push_str("- Cache tests include memory and filesystem caching\n");
        output.push_str("- Times include full pipeline: parsing, fetching, and compositing\n");

        std::fs::write(&output_path, output)?;
        println!("\n✅ Results saved to: {}", output_path);
    }

    println!("\n✨ Benchmarks complete!\n");

    Ok(())
}
