use crate::models::EmailMessage;
use anyhow::{anyhow, Context, Result};
use regex::Regex;
use reqwest::Client;
use serde_json::Value;

pub struct GraphApiClient {
    client: Client,
}

impl GraphApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_access_token(&self, client_id: &str, refresh_token: &str) -> Result<String> {
        println!(
            "[Graph API] Getting access token for client_id: {}",
            client_id
        );

        let url = "https://login.microsoftonline.com/common/oauth2/v2.0/token";

        let params = [
            ("client_id", client_id),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
            ("scope", "https://graph.microsoft.com/.default"),
        ];

        println!("[Graph API] Requesting access token from Microsoft OAuth2 endpoint");
        let response = self
            .client
            .post(url)
            .form(&params)
            .send()
            .await
            .context("Failed to send token request")?;

        let status = response.status();
        println!("[Graph API] OAuth2 response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            eprintln!("[Graph API] Failed to get access token: {}", error_text);
            return Err(anyhow!("Token request failed: {}", error_text));
        }

        let json: Value = response
            .json()
            .await
            .context("Failed to parse token response")?;

        let access_token = json
            .get("access_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("No access token in response"))?;

        println!("[Graph API] Successfully obtained access token");
        Ok(access_token)
    }

    pub async fn fetch_recent_emails(
        &self,
        access_token: &str,
        email: &str,
        max_results: usize,
    ) -> Result<Vec<EmailMessage>> {
        println!("[Graph API] Fetching recent emails for: {}", email);

        let url = format!("https://graph.microsoft.com/v1.0/users/{}/messages", email);

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .query(&[
                ("$top", max_results.to_string().as_str()),
                ("$select", "id,subject,from,toRecipients,receivedDateTime,sentDateTime,isRead,importance,body"),
                ("$orderby", "receivedDateTime desc"),
            ])
            .send()
            .await
            .context("Failed to fetch emails")?;

        let status = response.status();
        println!("[Graph API] Fetch emails response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            eprintln!("[Graph API] Email fetch failed: {}", error_text);
            return Err(anyhow!("Email fetch failed: {}", error_text));
        }

        let json: Value = response
            .json()
            .await
            .context("Failed to parse email response")?;

        let messages = json
            .get("value")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("No messages in response"))?;

        println!("[Graph API] Found {} messages", messages.len());

        let email_messages: Vec<EmailMessage> = messages
            .iter()
            .filter_map(|msg| {
                Some(EmailMessage {
                    id: msg.get("id")?.as_str()?.to_string(),
                    received_datetime: msg.get("receivedDateTime")?.as_str()?.to_string(),
                    sent_datetime: msg.get("sentDateTime")?.as_str()?.to_string(),
                    subject: msg.get("subject")?.as_str()?.to_string(),
                    body_content: msg.get("body")?.get("content")?.as_str()?.to_string(),
                    from_address: msg
                        .get("from")?
                        .get("emailAddress")?
                        .get("address")?
                        .as_str()?
                        .to_string(),
                })
            })
            .collect();

        Ok(email_messages)
    }

    pub fn extract_verification_code(body: &str) -> Option<String> {
        // Remove HTML tags
        let re_html = Regex::new(r"<[^>]+>").ok()?;
        let plain_text = re_html.replace_all(body, " ");

        // Look for 6-digit verification codes
        let re_code = Regex::new(r"\b(\d{6})\b").ok()?;

        if let Some(captures) = re_code.captures(&plain_text) {
            return captures.get(1).map(|m| m.as_str().to_string());
        }

        None
    }

    pub async fn get_verification_code_from_recent_email(
        &self,
        client_id: &str,
        refresh_token: &str,
        email: &str,
    ) -> Result<String> {
        println!("[Graph API] Getting verification code from recent emails");

        // Get access token
        let access_token = self.get_access_token(client_id, refresh_token).await?;

        // Fetch recent emails
        let emails = self.fetch_recent_emails(&access_token, email, 5).await?;

        // Look for verification code in the most recent email
        for email_msg in emails {
            println!(
                "[Graph API] Checking email from: {}",
                email_msg.from_address
            );
            println!("[Graph API] Subject: {}", email_msg.subject);

            // Check if this is from AWS/Builder ID or kiro.dev
            if email_msg.from_address.contains("signin.aws")
                || email_msg.from_address.contains("no-reply@")
                || email_msg.from_address.contains("kiro.dev")
            {
                if let Some(code) = Self::extract_verification_code(&email_msg.body_content) {
                    println!("[Graph API] ✓ Found verification code: {}", code);
                    return Ok(code);
                } else {
                    println!("[Graph API] No verification code found in this email");
                }
            }
        }

        eprintln!("[Graph API] No verification code found in recent emails");
        Err(anyhow!("No verification code found in recent emails"))
    }

    pub async fn wait_for_verification_code(
        &self,
        client_id: &str,
        refresh_token: &str,
        email: &str,
        timeout_seconds: u64,
    ) -> Result<String> {
        println!(
            "[Graph API] Starting to wait for verification code for email: {}",
            email
        );
        println!("[Graph API] Timeout: {} seconds", timeout_seconds);

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_seconds);

        let mut attempt = 0;
        loop {
            attempt += 1;
            let elapsed = start.elapsed();

            println!(
                "[Graph API] Attempt #{} (elapsed: {}s)",
                attempt,
                elapsed.as_secs()
            );

            if elapsed > timeout {
                eprintln!("[Graph API] Timeout after {} seconds", elapsed.as_secs());
                return Err(anyhow!("Timeout waiting for verification code"));
            }

            match self
                .get_verification_code_from_recent_email(client_id, refresh_token, email)
                .await
            {
                Ok(code) => {
                    println!("[Graph API] Successfully retrieved verification code");
                    return Ok(code);
                }
                Err(e) => {
                    println!("[Graph API] No code found yet: {}", e);
                    println!("[Graph API] Waiting 5 seconds before retry...");
                    // Wait 5 seconds before checking again
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
}
