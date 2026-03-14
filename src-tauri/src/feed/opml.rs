#[derive(Debug, Clone)]
pub struct OpmlItem {
    pub title: String,
    pub xml_url: String,
    pub html_url: Option<String>,
    pub folder_name: Option<String>,
    pub folder_id: Option<i64>,
}

pub fn parse_opml(xml: &str) -> Result<Vec<OpmlItem>, Box<dyn std::error::Error>> {
    let mut items = Vec::new();
    // Simple XML parsing for OPML structure
    // OPML is a simple format: <outline> elements with text, xmlUrl, htmlUrl attributes

    let mut in_body = false;
    let mut current_folder: Option<String> = None;

    for line in xml.lines() {
        let trimmed = line.trim();

        if trimmed.contains("<body") {
            in_body = true;
            continue;
        }
        if trimmed.contains("</body") {
            break;
        }
        if !in_body {
            continue;
        }

        if trimmed.starts_with("<outline") {
            let xml_url = extract_attr(trimmed, "xmlUrl")
                .or_else(|| extract_attr(trimmed, "xmlurl"));
            let text = extract_attr(trimmed, "text")
                .or_else(|| extract_attr(trimmed, "title"));
            let html_url = extract_attr(trimmed, "htmlUrl")
                .or_else(|| extract_attr(trimmed, "htmlurl"));

            if let Some(url) = xml_url {
                // This is a feed
                items.push(OpmlItem {
                    title: text.unwrap_or_else(|| url.clone()),
                    xml_url: url,
                    html_url,
                    folder_name: current_folder.clone(),
                    folder_id: None, // resolved during import
                });
            } else if let Some(folder_name) = text {
                // This is a folder (outline without xmlUrl)
                if !trimmed.ends_with("/>") {
                    current_folder = Some(folder_name);
                }
            }
        } else if trimmed == "</outline>" {
            current_folder = None;
        }
    }

    Ok(items)
}

fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
    let search = format!("{}=\"", attr_name);
    if let Some(pos) = tag.find(&search) {
        let start = pos + search.len();
        if let Some(end) = tag[start..].find('"') {
            return Some(tag[start..start + end].to_string());
        }
    }

    // Try single quotes
    let search_sq = format!("{}='", attr_name);
    if let Some(pos) = tag.find(&search_sq) {
        let start = pos + search_sq.len();
        if let Some(end) = tag[start..].find('\'') {
            return Some(tag[start..start + end].to_string());
        }
    }

    None
}

pub fn generate_opml(feeds: &[(String, String, Option<String>, Option<String>)]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="2.0">
  <head>
    <title>Kublai Reader Subscriptions</title>
  </head>
  <body>
"#,
    );

    let mut current_folder: Option<&str> = None;

    for (title, feed_url, html_url, folder_name) in feeds {
        let folder = folder_name.as_deref();

        if folder != current_folder {
            if current_folder.is_some() {
                xml.push_str("    </outline>\n");
            }
            if let Some(f) = folder {
                xml.push_str(&format!("    <outline text=\"{}\">\n", escape_xml(f)));
            }
            current_folder = folder;
        }

        let indent = if folder.is_some() { "      " } else { "    " };
        xml.push_str(&format!(
            "{}<outline type=\"rss\" text=\"{}\" xmlUrl=\"{}\"{}/>",
            indent,
            escape_xml(title),
            escape_xml(feed_url),
            html_url.as_ref().map(|u| format!(" htmlUrl=\"{}\"", escape_xml(u))).unwrap_or_default(),
        ));
        xml.push('\n');
    }

    if current_folder.is_some() {
        xml.push_str("    </outline>\n");
    }

    xml.push_str("  </body>\n</opml>\n");
    xml
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
