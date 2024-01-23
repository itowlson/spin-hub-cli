use anyhow::anyhow;

pub async fn clone_decoupled(repo: &str) -> anyhow::Result<()> {
    let status = tokio::process::Command::new("git")
        .args(["clone", "-o", "upstream"])
        .arg(repo)
        .status()
        .await?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("git clone failed - see output for details"))
    }
}

pub fn clone_dir(repo: &str) -> anyhow::Result<String> {
    let url = url::Url::parse(repo)?;
    let path_segments = url.path_segments().ok_or(anyhow!("can't determine output directory"))?;
    let last_segment = path_segments.last().ok_or(anyhow!("can't determine output directory"))?;
    let dir = last_segment.strip_suffix(".git").unwrap_or(last_segment);
    Ok(dir.to_owned())
}
