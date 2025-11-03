use iocraft::prelude::*;
use taletrail_cli::{App, Config, Result};

fn main() -> Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Validate configuration
    config.validate()?;

    // Run the Iocraft application with smol async executor
    smol::block_on(async {
        element!(App)
            .render_loop()
            .await
            .map_err(|e| taletrail_cli::error::AppError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string()
            )))?;

        Ok(())
    })
}
