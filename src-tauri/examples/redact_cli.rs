use std::path::Path;
use std::time::Instant;

use karma_lib::inference::InferenceEngine;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let model_dir = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("/Users/chen/.lmstudio/models/mlx-community/openai-privacy-filter-bf16");
    let baseline_path = args
        .get(2)
        .map(String::as_str)
        .unwrap_or("/tmp/py_redact_baseline.json");
    let iters: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(20);

    let baseline: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(baseline_path)?)?;
    let text = baseline["text"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("baseline missing text field"))?
        .to_string();

    let engine = InferenceEngine::new();
    let t0 = Instant::now();
    let info = engine.load_model(Path::new(model_dir))?;
    println!(
        "load: {:.2}s model={} arch={} labels={}",
        t0.elapsed().as_secs_f64(),
        info.name,
        info.architecture,
        info.num_labels
    );

    let t1 = Instant::now();
    let result = engine.redact(&text)?;
    println!(
        "cold redact: {:.1}ms (reported {:.2}ms) spans={}",
        t1.elapsed().as_secs_f64() * 1000.0,
        result.latency_ms,
        result.detected_spans.len()
    );

    let mut lat = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t = Instant::now();
        engine.redact(&text)?;
        lat.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    lat.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!(
        "warm: min={:.1} p50={:.1} max={:.1} ms",
        lat[0],
        lat[lat.len() / 2],
        lat[lat.len() - 1]
    );

    std::fs::write(
        "/tmp/rust_redact_output.json",
        serde_json::to_string_pretty(&result)?,
    )?;
    println!("wrote /tmp/rust_redact_output.json");
    Ok(())
}
