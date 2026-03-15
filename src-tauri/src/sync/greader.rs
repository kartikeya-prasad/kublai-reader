use super::{SyncEntry, SyncFeed, SyncResult};
use serde_json::Value;

pub struct GoogleReaderClient {
    pub server_url: String,
    pub username: String,
    pub auth_token: Option<String>,
    client: reqwest::Client,
}

impl GoogleReaderClient {
    pub fn new(server_url: String, username: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("KublaiReader/0.2")
            .build()
            .unwrap_or_default();
        Self { server_url, username, auth_token: None, client }
    }

    pub async fn login(&mut self, password: &str) -> Result<(), String> {
        let url = format!("{}/accounts/ClientLogin", self.server_url.trim_end_matches('/'));
        let params = [
            ("Email", self.username.as_str()),
            ("Passwd", password),
            ("service", "reader"),
            ("accountType", "HOSTED_OR_GOOGLE"),
        ];
        let resp = self.client.post(&url)
            .form(&params)
            .send().await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Login failed: {}", resp.status()));
        }

        let body = resp.text().await.map_err(|e| e.to_string())?;
        for line in body.lines() {
            if let Some(token) = line.strip_prefix("Auth=") {
                self.auth_token = Some(token.to_string());
                return Ok(());
            }
        }
        Err("No Auth token in response".to_string())
    }

    fn auth_header(&self) -> Result<String, String> {
        self.auth_token.as_ref()
            .map(|t| format!("GoogleLogin auth={}", t))
            .ok_or_else(|| "Not authenticated".to_string())
    }

    pub async fn get_subscriptions(&self) -> Result<Vec<SyncFeed>, String> {
        let url = format!("{}/reader/api/0/subscription/list?output=json",
            self.server_url.trim_end_matches('/'));
        let resp = self.client.get(&url)
            .header("Authorization", self.auth_header()?)
            .send().await.map_err(|e| e.to_string())?;

        let json: Value = resp.json::<Value>().await.map_err(|e| e.to_string())?;
        let mut feeds = Vec::new();
        if let Some(subs) = json["subscriptions"].as_array() {
            for sub in subs {
                let id = sub["id"].as_str().unwrap_or("").to_string();
                let title = sub["title"].as_str().unwrap_or("").to_string();
                let url = sub["url"].as_str().unwrap_or("").to_string();
                let site_url = sub["htmlUrl"].as_str().unwrap_or("").to_string();
                let category = sub["categories"].as_array()
                    .and_then(|c: &Vec<Value>| c.first())
                    .and_then(|c: &Value| c["label"].as_str())
                    .map(|s: &str| s.to_string());
                if !url.is_empty() {
                    feeds.push(SyncFeed { id, title, url, site_url, category });
                }
            }
        }
        Ok(feeds)
    }

    pub async fn get_unread_ids(&self) -> Result<Vec<String>, String> {
        let url = format!(
            "{}/reader/api/0/stream/items/ids?output=json&s=user/-/state/com.google/reading-list&xt=user/-/state/com.google/read&n=10000",
            self.server_url.trim_end_matches('/')
        );
        let resp = self.client.get(&url)
            .header("Authorization", self.auth_header()?)
            .send().await.map_err(|e| e.to_string())?;
        let json: Value = resp.json::<Value>().await.map_err(|e| e.to_string())?;
        Ok(json["itemRefs"].as_array().unwrap_or(&vec![])
            .iter()
            .filter_map(|i| i["id"].as_str().map(|s: &str| s.to_string()))
            .collect())
    }

    pub async fn mark_read(&self, ids: &[String]) -> Result<(), String> {
        let url = format!("{}/reader/api/0/edit-tag", self.server_url.trim_end_matches('/'));
        let token = self.get_token().await?;
        let mut params = vec![
            ("a".to_string(), "user/-/state/com.google/read".to_string()),
            ("T".to_string(), token),
        ];
        for id in ids {
            params.push(("i".to_string(), id.clone()));
        }
        self.client.post(&url)
            .header("Authorization", self.auth_header()?)
            .form(&params)
            .send().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn mark_starred(&self, ids: &[String]) -> Result<(), String> {
        let url = format!("{}/reader/api/0/edit-tag", self.server_url.trim_end_matches('/'));
        let token = self.get_token().await?;
        let mut params = vec![
            ("a".to_string(), "user/-/state/com.google/starred".to_string()),
            ("T".to_string(), token),
        ];
        for id in ids {
            params.push(("i".to_string(), id.clone()));
        }
        self.client.post(&url)
            .header("Authorization", self.auth_header()?)
            .form(&params)
            .send().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get_token(&self) -> Result<String, String> {
        let url = format!("{}/reader/api/0/token", self.server_url.trim_end_matches('/'));
        let resp = self.client.get(&url)
            .header("Authorization", self.auth_header()?)
            .send().await.map_err(|e| e.to_string())?;
        resp.text().await.map_err(|e| e.to_string())
    }
}

// Suppress unused import warnings for stub types
#[allow(unused)]
fn _use_sync_entry(_: SyncEntry) {}
#[allow(unused)]
fn _use_sync_result(_: SyncResult) {}
