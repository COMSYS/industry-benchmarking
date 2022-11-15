//! MAIN SERVER ENTRY POINT

use server::execute_server;
#[cfg(feature="evaluation")]
use chrono::prelude::*;

/// Execute server as binary package
fn main() -> std::io::Result<()> {
    
    
    #[cfg(feature="evaluation")]
    println!("{:?}\n== Evaluation Enabled! ==", Local::now());
    
    #[cfg(not(feature="evaluation"))]
    {
        // Start logger and set if unconfigured
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
        log::info!("Running in debug mode!");
    }

    execute_server()
}