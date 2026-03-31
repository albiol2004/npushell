use crate::config::ApiConfig;
use serde_json::json;
use std::time::Duration;

pub struct CopilotClient {
    http: reqwest::blocking::Client,
    endpoint: String,
    api_path: String,
    model: String,
}

impl CopilotClient {
    pub fn new(config: &ApiConfig) -> Self {
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .expect("failed to build HTTP client");

        Self {
            http,
            endpoint: config.endpoint.clone(),
            api_path: config.api_path.clone(),
            model: config.model.clone(),
        }
    }

    pub fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}{}/chat/completions", self.endpoint, self.api_path);

        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt },
            ],
            "temperature": 0.3,
            "max_tokens": 1024,
        });

        let response = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().unwrap_or_default();
            return Err(format!("API request failed ({}): {}", status, text).into());
        }

        let data: serde_json::Value = response.json()?;

        let content = data["choices"]
            .get(0)
            .and_then(|c| c["message"]["content"].as_str())
            .ok_or("unexpected API response: missing choices[0].message.content")?;

        Ok(content.to_string())
    }

    /// Check if the API endpoint is reachable.
    pub fn health_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}{}/models", self.endpoint, self.api_path);
        let response = self.http.get(&url).send()?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("API returned status {}", response.status()).into())
        }
    }
}
