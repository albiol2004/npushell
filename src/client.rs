use crate::config::ApiConfig;
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
        max_tokens: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}{}/chat/completions", self.endpoint, self.api_path);

        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt },
            ],
            "temperature": 0.3,
            "max_tokens": max_tokens,
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

    /// Stream a completion, printing tokens to stdout as they arrive.
    /// Returns the full accumulated response.
    pub fn complete_streaming(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}{}/chat/completions", self.endpoint, self.api_path);

        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt },
            ],
            "temperature": 0.3,
            "max_tokens": max_tokens,
            "stream": true,
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

        // Start spinner while waiting for first token
        let spinning = Arc::new(AtomicBool::new(true));
        let spinning_clone = spinning.clone();
        let spinner_handle = std::thread::spawn(move || {
            let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let mut i = 0;
            let stderr = std::io::stderr();
            while spinning_clone.load(Ordering::Relaxed) {
                let mut err = stderr.lock();
                let _ = write!(err, "\r\x1b[2m{} thinking...\x1b[0m", frames[i % frames.len()]);
                let _ = err.flush();
                i += 1;
                std::thread::sleep(Duration::from_millis(80));
            }
            // Clear spinner line
            let mut err = stderr.lock();
            let _ = write!(err, "\r\x1b[2K");
            let _ = err.flush();
        });

        let reader = BufReader::new(response);
        let mut full_response = String::new();
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut spinner_handle = Some(spinner_handle);

        for line in reader.lines() {
            let line = line?;
            let Some(data) = line.strip_prefix("data: ") else {
                continue;
            };

            if data == "[DONE]" {
                break;
            }

            if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(content) = chunk["choices"]
                    .get(0)
                    .and_then(|c| c["delta"]["content"].as_str())
                {
                    if let Some(handle) = spinner_handle.take() {
                        spinning.store(false, Ordering::Relaxed);
                        let _ = handle.join();
                    }
                    full_response.push_str(content);
                    write!(out, "{}", content)?;
                    out.flush()?;
                }
            }
        }

        if let Some(handle) = spinner_handle.take() {
            spinning.store(false, Ordering::Relaxed);
            let _ = handle.join();
        }

        writeln!(out)?;
        Ok(full_response)
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
