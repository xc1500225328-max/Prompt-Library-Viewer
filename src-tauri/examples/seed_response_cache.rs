#[path = "../src/parser.rs"]
mod parser;

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let cache_dir = args
        .next()
        .map(std::path::PathBuf::from)
        .expect("missing cache dir");
    let force_refresh = args.any(|arg| arg == "--refresh");

    let started = std::time::Instant::now();
    let response = parser::load_and_parse_prompts(cache_dir.clone(), force_refresh)
        .await
        .expect("failed to seed response cache");

    println!(
        "seeded cache={} items={} sources={} categories={} errors={} elapsed_ms={}",
        cache_dir.display(),
        response.items.len(),
        response.sources.len(),
        response.categories.len(),
        response.errors.len(),
        started.elapsed().as_millis()
    );
}
