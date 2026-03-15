mod cache;
mod executor;
mod model;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    let cache_dir = cache::get_cache_dir();
    println!("Scanning cache directory: {:?}", cache_dir);

    let mut models = cache::scan_cache(&cache_dir)?;

    // Sort models alphabetically by name
    models.sort_by(|a, b| a.name.cmp(&b.name));

    if models.is_empty() {
        eprintln!("No models found in cache directory");
        std::process::exit(1);
    }

    println!("Found {} models", models.len());

    let mut ui = ui::Ui::new(models);

    let selected_model = ui.run()?;

    if let Some(model) = selected_model {
        let params = cache::load_params(&cache_dir)?;
        executor::launch_server(&model, &params)?;
    } else {
        println!("No model selected");
    }

    Ok(())
}
