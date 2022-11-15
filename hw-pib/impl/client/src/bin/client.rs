use client::{execute_client, config};

#[tokio::main]

/// Simple CLI enabled client that uses the external
/// client lib for starting it up.
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start logger and set if unconfigured
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Parse CLI arguments
    let config = config::configure_client_application();

    // Start lib client
    execute_client(config).await
}