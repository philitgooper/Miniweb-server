use miniweb_server::{config::Config, logging};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("miniweb-server: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::load()?;
    logging::init(&config.log_level)?;
    miniweb_server::server::run(config).await?;
    Ok(())
}
