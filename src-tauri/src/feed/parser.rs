pub fn parse(
    bytes: &[u8],
    url: Option<&str>,
) -> Result<feed_rs::model::Feed, Box<dyn std::error::Error + Send + Sync>> {
    let feed = feed_rs::parser::Builder::new()
        .base_uri(url)
        .build()
        .parse(bytes)?;
    Ok(feed)
}
