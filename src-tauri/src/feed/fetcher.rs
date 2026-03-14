use reqwest::StatusCode;

/// Fetch a feed URL with conditional GET support.
/// Returns None if 304 Not Modified.
/// Returns (parsed_feed, new_etag, new_last_modified) if content was fetched.
pub async fn fetch_feed(
    url: &str,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> Result<Option<(feed_rs::model::Feed, Option<String>, Option<String>)>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) KublaiReader/0.1")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let mut request = client.get(url);

    if let Some(etag) = etag {
        request = request.header("If-None-Match", etag);
    }
    if let Some(lm) = last_modified {
        request = request.header("If-Modified-Since", lm);
    }

    let response = request.send().await?;

    if response.status() == StatusCode::NOT_MODIFIED {
        return Ok(None);
    }

    if !response.status().is_success() {
        return Err(format!("HTTP {}: {}", response.status(), url).into());
    }

    let new_etag = response.headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let new_last_modified = response.headers()
        .get("last-modified")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let bytes = response.bytes().await?;
    let parsed = super::parser::parse(&bytes, Some(url))?;

    Ok(Some((parsed, new_etag, new_last_modified)))
}

/// Fetch and auto-detect a feed from a URL.
/// If the URL is an HTML page, look for <link rel="alternate"> tags.
/// Returns (feed_url, parsed_feed).
pub async fn fetch_and_detect(
    url: &str,
) -> Result<(String, feed_rs::model::Feed), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) KublaiReader/0.1")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()).into());
    }

    let content_type = response.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    let bytes = response.bytes().await?;

    // Try parsing directly as a feed
    if content_type.contains("xml") || content_type.contains("rss") || content_type.contains("atom") || content_type.contains("json") {
        match super::parser::parse(&bytes, Some(url)) {
            Ok(feed) => return Ok((url.to_string(), feed)),
            Err(_) => {} // Fall through to HTML detection
        }
    }

    // Also try parsing even if content type is HTML (some servers misconfigure)
    if let Ok(feed) = super::parser::parse(&bytes, Some(url)) {
        return Ok((url.to_string(), feed));
    }

    // Parse as HTML and look for feed links
    let html = String::from_utf8_lossy(&bytes);
    let feed_urls = detect_feed_links(&html, url);

    for feed_url in feed_urls {
        let resp = client.get(&feed_url).send().await?;
        if resp.status().is_success() {
            let feed_bytes = resp.bytes().await?;
            if let Ok(feed) = super::parser::parse(&feed_bytes, Some(&feed_url)) {
                return Ok((feed_url, feed));
            }
        }
    }

    Err("No RSS/Atom feed found at this URL".into())
}

fn detect_feed_links(html: &str, base_url: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let base = url::Url::parse(base_url).ok();

    // Simple regex-free detection: look for link tags with type="application/rss+xml" or type="application/atom+xml"
    let html_lower = html.to_lowercase();
    for link_type in &["application/rss+xml", "application/atom+xml", "application/feed+json"] {
        if let Some(pos) = html_lower.find(link_type) {
            // Find the enclosing <link> tag
            if let Some(tag_start) = html_lower[..pos].rfind("<link") {
                let tag_end = html_lower[tag_start..].find('>').unwrap_or(html_lower.len() - tag_start);
                let tag = &html[tag_start..tag_start + tag_end];

                // Extract href attribute
                if let Some(href_pos) = tag.to_lowercase().find("href=") {
                    let href_start = href_pos + 5;
                    let quote = tag.as_bytes().get(href_start).copied().unwrap_or(b'"');
                    let delim = if quote == b'"' || quote == b'\'' { quote as char } else { ' ' };
                    let href_content_start = if delim == '"' || delim == '\'' { href_start + 1 } else { href_start };
                    let href_end = tag[href_content_start..].find(delim).unwrap_or(tag.len() - href_content_start);
                    let href = &tag[href_content_start..href_content_start + href_end];

                    if let Some(ref base) = base {
                        if let Ok(resolved) = base.join(href) {
                            urls.push(resolved.to_string());
                        }
                    } else {
                        urls.push(href.to_string());
                    }
                }
            }
        }
    }

    // Try common feed paths as fallback
    if urls.is_empty() {
        if let Some(ref base) = base {
            for path in &["/feed", "/rss", "/atom.xml", "/feed.xml", "/rss.xml", "/index.xml"] {
                if let Ok(resolved) = base.join(path) {
                    urls.push(resolved.to_string());
                }
            }
        }
    }

    urls
}
