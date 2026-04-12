pub mod extractor;

/// Fetch a URL and extract readable article content.
/// Shared by the manual parser command and the background auto-parse scheduler.
pub async fn fetch_and_extract(url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let url = url.to_string();
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) KublaiReader/0.3")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let html = client.get(&url).send().await?.text().await?;

    let url_clone = url.clone();
    let extracted = tokio::task::spawn_blocking(move || {
        extractor::extract(&html, &url_clone)
    })
    .await??;

    Ok(extracted.content)
}
