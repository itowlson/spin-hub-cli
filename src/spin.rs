pub fn version() -> String {
    std::env::var("SPIN_VERSION").unwrap_or_else(|_| "2.0.0".to_owned())
}
