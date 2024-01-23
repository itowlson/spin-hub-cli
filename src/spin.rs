pub fn version() -> String {
    std::env::var("SPIN_VERSION").unwrap_or_else(|_| "2.0.0".to_owned())
}

pub fn bin() -> tokio::process::Command {
    tokio::process::Command::new(std::env::var("SPIN_BIN_PATH").unwrap())
}
