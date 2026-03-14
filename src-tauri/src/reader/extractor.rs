/// Extracted article content from readability parsing.
pub struct ExtractedArticle {
    pub title: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub author: Option<String>,
}

/// Extract readable article content from HTML using a simple heuristic approach.
/// Note: readabilityrs will be integrated once its API is confirmed compatible.
/// For now, we use a basic content extraction that strips navigation/ads.
pub fn extract(html: &str, _url: &str) -> Result<ExtractedArticle, Box<dyn std::error::Error + Send + Sync>> {
    // Basic content extraction:
    // 1. Find <article> or <main> or role="main" content
    // 2. Strip <script>, <style>, <nav>, <header>, <footer>, <aside> tags
    // 3. Return the cleaned HTML

    let mut content = html.to_string();

    // Remove script and style tags with their content
    content = remove_tags_with_content(&content, "script");
    content = remove_tags_with_content(&content, "style");
    content = remove_tags_with_content(&content, "nav");
    content = remove_tags_with_content(&content, "footer");
    content = remove_tags_with_content(&content, "aside");

    // Try to find the main content area
    let main_content = find_content_block(&content);

    // Extract title from <title> tag or <h1>
    let title = extract_tag_content(&content, "title")
        .or_else(|| extract_tag_content(&content, "h1"))
        .unwrap_or_default();

    // Generate a short excerpt from the first paragraph
    let excerpt = extract_tag_content(&main_content, "p");

    Ok(ExtractedArticle {
        title,
        content: main_content,
        excerpt,
        author: None,
    })
}

fn remove_tags_with_content(html: &str, tag: &str) -> String {
    let mut result = html.to_string();
    let open_tag = format!("<{}", tag);
    let close_tag = format!("</{}>", tag);

    loop {
        let lower = result.to_lowercase();
        if let Some(start) = lower.find(&open_tag) {
            if let Some(end) = lower[start..].find(&close_tag) {
                let remove_end = start + end + close_tag.len();
                result = format!("{}{}", &result[..start], &result[remove_end..]);
                continue;
            }
        }
        break;
    }

    result
}

fn find_content_block(html: &str) -> String {
    let lower = html.to_lowercase();

    // Try <article> first
    if let Some(start) = lower.find("<article") {
        if let Some(end) = lower[start..].find("</article>") {
            return html[start..start + end + 10].to_string();
        }
    }

    // Try <main>
    if let Some(start) = lower.find("<main") {
        if let Some(end) = lower[start..].find("</main>") {
            return html[start..start + end + 7].to_string();
        }
    }

    // Try role="main"
    if let Some(start) = lower.find("role=\"main\"") {
        if let Some(tag_start) = lower[..start].rfind('<') {
            // Find the corresponding closing tag
            if let Some(tag_name_end) = lower[tag_start + 1..].find(|c: char| c.is_whitespace() || c == '>') {
                let tag_name = &html[tag_start + 1..tag_start + 1 + tag_name_end];
                let close = format!("</{}>", tag_name.to_lowercase());
                if let Some(end) = lower[start..].find(&close) {
                    return html[tag_start..start + end + close.len()].to_string();
                }
            }
        }
    }

    // Fallback: return <body> content
    if let Some(start) = lower.find("<body") {
        if let Some(body_start) = lower[start..].find('>') {
            let content_start = start + body_start + 1;
            if let Some(end) = lower[content_start..].find("</body>") {
                return html[content_start..content_start + end].to_string();
            }
        }
    }

    html.to_string()
}

fn extract_tag_content(html: &str, tag: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);

    if let Some(start) = lower.find(&open) {
        if let Some(tag_end) = lower[start..].find('>') {
            let content_start = start + tag_end + 1;
            if let Some(end) = lower[content_start..].find(&close) {
                let content = &html[content_start..content_start + end];
                // Strip HTML tags from content for plain text
                let plain = strip_tags(content);
                if !plain.trim().is_empty() {
                    return Some(plain.trim().to_string());
                }
            }
        }
    }

    None
}

fn strip_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }

    result
}
