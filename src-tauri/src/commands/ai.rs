use crate::db::AppDatabase;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[tauri::command]
pub async fn summarize_article(
    article_id: i64,
    db: State<'_, AppDatabase>,
) -> Result<String, String> {
    // Load article content and settings
    let (article_text, api_key, model, cached_summary) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        let (text, cached): (Option<String>, Option<String>) = conn.query_row(
            "SELECT COALESCE(parsed_content, content, summary), ai_summary FROM articles WHERE id = ?1",
            [article_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|e| format!("Article not found: {}", e))?;

        let key: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'openrouter_api_key'",
            [],
            |row| row.get(0),
        ).unwrap_or_default();

        let mdl: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'openrouter_model'",
            [],
            |row| row.get(0),
        ).unwrap_or_else(|_| "anthropic/claude-haiku-4-5".to_string());

        (text.unwrap_or_default(), key, mdl, cached)
    };

    // Return cached summary if available
    if let Some(summary) = cached_summary {
        if !summary.is_empty() {
            return Ok(summary);
        }
    }

    if api_key.is_empty() {
        return Err("No OpenRouter API key configured. Add it in Settings → AI.".to_string());
    }

    if article_text.is_empty() {
        return Err("No article content available to summarize.".to_string());
    }

    // Truncate content to ~4000 chars to stay within token limits
    let content_snippet: String = article_text.chars().take(4000).collect();

    // Strip HTML tags for cleaner input
    let plain_text = strip_html(&content_snippet);

    let client = reqwest::Client::new();
    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .header(CONTENT_TYPE, "application/json")
        .header("HTTP-Referer", "https://kublai-reader.app")
        .header("X-Title", "Kublai Reader")
        .json(&json!({
            "model": model,
            "messages": [{
                "role": "user",
                "content": format!(
                    "Summarize the following article in 2-3 concise sentences. Focus on the key points and main takeaway. Do not include any preamble like 'This article discusses'.\n\n{}",
                    plain_text
                )
            }],
            "max_tokens": 200
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to contact OpenRouter: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("OpenRouter error {}: {}", status, body));
    }

    let chat: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let summary = chat
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| "Empty response from AI".to_string())?;

    // Cache in DB
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE articles SET ai_summary = ?1 WHERE id = ?2",
            rusqlite::params![summary, article_id],
        ).map_err(|e| e.to_string())?;
    }

    Ok(summary)
}

fn strip_html(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;
    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(c),
            _ => {}
        }
    }
    // Collapse whitespace
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}
