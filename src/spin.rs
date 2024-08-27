pub fn version() -> String {
    std::env::var("SPIN_VERSION").unwrap_or_else(|_| "2.0.0".to_owned())
}

pub fn bin() -> anyhow::Result<tokio::process::Command> {
    let bin_path = std::env::var("SPIN_BIN_PATH")
        .map_err(|_| anyhow::anyhow!("Environment variable SPIN_BIN_PATH not set"))?;
    
    Ok(tokio::process::Command::new(bin_path))
}
