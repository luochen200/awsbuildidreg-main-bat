use anyhow::{anyhow, Context, Result};
use chrono::{Duration, Utc};
use mailparse::parse_mail;
use regex::Regex;

pub struct ImapClient {
    imap_server: String,
    imap_port: u16,
}

impl ImapClient {
    pub fn new() -> Self {
        Self {
            imap_server: "outlook.office365.com".to_string(),
            imap_port: 993,
        }
    }

    /// Get access token from refresh token using OAuth2
    async fn get_access_token(client_id: &str, refresh_token: &str) -> Result<String> {
        println!("[IMAP] Getting access token for client_id: {}", client_id);
        println!("[IMAP] Refresh token length: {} chars", refresh_token.len());
        println!(
            "[IMAP] Refresh token prefix: {}...",
            &refresh_token[..std::cmp::min(20, refresh_token.len())]
        );

        let client = reqwest::Client::new();

        // Use the correct scope format for IMAP - note: outlook.office.com not office365.com
        let params = [
            ("client_id", client_id),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
            (
                "scope",
                "https://outlook.office.com/IMAP.AccessAsUser.All offline_access",
            ),
        ];

        println!("[IMAP] Requesting access token from Microsoft OAuth2 endpoint");
        println!(
            "[IMAP] Using scope: https://outlook.office.com/IMAP.AccessAsUser.All offline_access"
        );

        let response = client
            .post("https://login.microsoftonline.com/common/oauth2/v2.0/token")
            .form(&params)
            .send()
            .await
            .context("Failed to request access token")?;

        let status = response.status();
        println!("[IMAP] OAuth2 response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            eprintln!("[IMAP] Failed to get access token: {}", error_text);
            return Err(anyhow!("Failed to get access token: {}", error_text));
        }

        let json: serde_json::Value = response.json().await?;

        // Check if we got a new refresh token
        if let Some(new_refresh_token) = json.get("refresh_token").and_then(|v| v.as_str()) {
            println!(
                "[IMAP] Received new refresh_token (length: {})",
                new_refresh_token.len()
            );
        }

        let access_token = json
            .get("access_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("No access_token in response"))?;

        println!(
            "[IMAP] Successfully obtained access token (length: {})",
            access_token.len()
        );
        println!(
            "[IMAP] Access token prefix: {}...",
            &access_token[..std::cmp::min(50, access_token.len())]
        );

        Ok(access_token)
    }

    /// Extract verification code from email body (text or HTML)
    fn extract_verification_code(body: &str) -> Option<String> {
        // Remove HTML tags first (like Graph API does)
        let re_html = Regex::new(r"<[^>]+>").ok()?;
        let plain_text = re_html.replace_all(body, " ");

        // Look for 6-digit verification codes
        let re_code = Regex::new(r"\b(\d{6})\b").ok()?;

        if let Some(captures) = re_code.captures(&plain_text) {
            return captures.get(1).map(|m| m.as_str().to_string());
        }

        None
    }

    /// Extract both text and HTML from email
    fn extract_email_content(parsed: &mailparse::ParsedMail) -> String {
        let mut content = String::new();

        // Try to get plain text body
        if let Ok(text) = parsed.get_body() {
            content.push_str(&text);
            content.push('\n');
        }

        // Also try to get HTML from subparts
        for subpart in &parsed.subparts {
            if let Ok(part_text) = subpart.get_body() {
                content.push_str(&part_text);
                content.push('\n');
            }

            // Recursively check nested parts
            if !subpart.subparts.is_empty() {
                content.push_str(&Self::extract_email_content(subpart));
            }
        }

        content
    }

    /// Fetch recent emails using raw IMAP commands with SASL-IR authentication
    /// This implementation follows the same pattern as the Python code
    async fn fetch_recent_emails(
        &self,
        email: &str,
        access_token: &str,
        mailbox: &str,
        limit: usize,
    ) -> Result<Vec<(String, String)>> {
        use async_native_tls::TlsConnector;
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        use futures::io::BufReader;
        use futures::{AsyncBufReadExt, AsyncWriteExt};
        use tokio::net::TcpStream;
        use tokio_util::compat::TokioAsyncReadCompatExt;

        println!(
            "[IMAP] Connecting to {}:{}",
            self.imap_server, self.imap_port
        );

        // Note: If using TUN mode proxy, IMAP SASL authentication might fail
        // Consider adding direct connection rules for outlook.office365.com

        // Connect to IMAP server
        let tcp_stream = TcpStream::connect((&self.imap_server[..], self.imap_port))
            .await
            .context("Failed to connect to IMAP server")?;

        println!("[IMAP] TCP connection established, starting TLS handshake");

        // Convert tokio stream to futures-compatible stream
        let compat_stream = tcp_stream.compat();

        let tls = TlsConnector::new();
        let tls_stream = tls
            .connect(&self.imap_server, compat_stream)
            .await
            .context("Failed to establish TLS connection")?;

        println!("[IMAP] TLS connection established");

        // Split the stream for reading and writing
        let (reader, mut writer) = futures::io::AsyncReadExt::split(tls_stream);
        let mut reader = BufReader::new(reader);

        // Read server greeting
        let mut greeting = String::new();
        reader
            .read_line(&mut greeting)
            .await
            .context("Failed to read greeting")?;
        println!("[IMAP] Server greeting: {}", greeting.trim());

        // Build XOAUTH2 auth string and base64 encode it (SASL-IR style)
        let auth_string = format!("user={}\x01auth=Bearer {}\x01\x01", email, access_token);
        let auth_string_b64 = STANDARD.encode(&auth_string);

        println!(
            "[IMAP] Authenticating with XOAUTH2 SASL-IR for user: {}",
            email
        );
        println!(
            "[IMAP] Token length: {} chars, Auth base64 length: {} chars",
            access_token.len(),
            auth_string_b64.len()
        );

        // Send AUTHENTICATE command with SASL-IR (initial response included)
        // Format: A1 AUTHENTICATE XOAUTH2 <base64_auth_string>
        let auth_command = format!("A1 AUTHENTICATE XOAUTH2 {}\r\n", auth_string_b64);
        println!(
            "[IMAP] Sending: A1 AUTHENTICATE XOAUTH2 <base64 {} chars>",
            auth_string_b64.len()
        );

        writer
            .write_all(auth_command.as_bytes())
            .await
            .context("Failed to send auth command")?;
        writer.flush().await.context("Failed to flush")?;

        // Read authentication response
        let mut auth_response = String::new();
        reader
            .read_line(&mut auth_response)
            .await
            .context("Failed to read auth response")?;
        println!("[IMAP] Auth response: {}", auth_response.trim());

        // Check if we need to handle a challenge (+ response)
        if auth_response.starts_with('+') {
            // Server is asking for more data, send empty response to abort or continue
            // For XOAUTH2, if we get here with SASL-IR, it usually means an error
            println!("[IMAP] Received challenge, sending empty response");
            writer.write_all(b"\r\n").await?;
            writer.flush().await?;

            // Read the final response
            auth_response.clear();
            reader.read_line(&mut auth_response).await?;
            println!("[IMAP] Final response: {}", auth_response.trim());
        }

        // Check if authentication succeeded
        if !auth_response.contains(" OK ") {
            return Err(anyhow!(
                "IMAP authentication failed: {}",
                auth_response.trim()
            ));
        }

        println!("[IMAP] Authentication successful!");

        // Reassemble the stream for async_imap Session
        // We need to create a new Client with the authenticated stream
        let combined_stream = reader
            .into_inner()
            .reunite(writer)
            .map_err(|_| anyhow!("Failed to reunite stream"))?;

        // Create session from the authenticated connection
        // Note: We can't easily create an async_imap::Session from a raw stream
        // So we'll continue using raw commands for simplicity

        // Select mailbox (INBOX or Junk)
        let select_cmd = format!("A2 SELECT {}\r\n", mailbox);
        let (reader, mut writer) = futures::io::AsyncReadExt::split(combined_stream);
        let mut reader = BufReader::new(reader);

        writer.write_all(select_cmd.as_bytes()).await?;
        writer.flush().await?;

        // Read SELECT response (multiple lines until we get A2 OK/NO/BAD)
        let mut exists: u32 = 0;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            println!("[IMAP] SELECT response: {}", line.trim());

            // Parse EXISTS count
            if line.contains(" EXISTS") {
                if let Some(count_str) = line.split_whitespace().nth(1) {
                    exists = count_str.parse().unwrap_or(0);
                }
            }

            if line.starts_with("A2 ") {
                if !line.contains(" OK ") {
                    return Err(anyhow!("Failed to select {}: {}", mailbox, line.trim()));
                }
                break;
            }
        }

        println!("[IMAP] {} selected, {} messages", mailbox, exists);

        let mut emails = Vec::new();

        if exists > 0 {
            // Calculate range for fetching
            let start_seq = if exists > limit as u32 {
                exists - limit as u32 + 1
            } else {
                1
            };
            let range = format!("{}:*", start_seq);

            // Fetch messages
            let fetch_cmd = format!("A3 FETCH {} RFC822\r\n", range);
            println!("[IMAP] Fetching messages: {}", range);

            // Reunite stream for fetch
            let combined = reader
                .into_inner()
                .reunite(writer)
                .map_err(|_| anyhow!("Failed to reunite stream"))?;
            let (reader, mut writer) = futures::io::AsyncReadExt::split(combined);
            let mut reader = BufReader::new(reader);

            writer.write_all(fetch_cmd.as_bytes()).await?;
            writer.flush().await?;

            // Parse FETCH responses (complex multi-line format)
            let mut current_body = Vec::new();
            let mut in_literal = false;
            let mut literal_remaining = 0usize;

            loop {
                let mut line = String::new();
                if in_literal {
                    // Read literal data
                    let mut buf = vec![0u8; literal_remaining.min(8192)];
                    let n = futures::io::AsyncReadExt::read(&mut reader, &mut buf).await?;
                    current_body.extend_from_slice(&buf[..n]);
                    literal_remaining -= n;

                    if literal_remaining == 0 {
                        in_literal = false;
                        // Parse the email
                        if let Ok(parsed) = parse_mail(&current_body) {
                            let from = parsed
                                .headers
                                .iter()
                                .find(|h| h.get_key().eq_ignore_ascii_case("From"))
                                .map(|h| h.get_value())
                                .unwrap_or_else(|| "Unknown".to_string());

                            // Extract all content (text + HTML)
                            let body_text = Self::extract_email_content(&parsed);

                            emails.push((from, body_text));
                        }
                        current_body.clear();
                    }
                } else {
                    reader.read_line(&mut line).await?;

                    // Check for literal start: * n FETCH (... {size}
                    if line.contains("{") && line.contains("}") {
                        if let Some(start) = line.rfind('{') {
                            if let Some(end) = line.rfind('}') {
                                if let Ok(size) = line[start + 1..end].parse::<usize>() {
                                    literal_remaining = size;
                                    in_literal = true;
                                    current_body.clear();
                                }
                            }
                        }
                    }

                    if line.starts_with("A3 ") {
                        break;
                    }
                }
            }

            // Reunite and logout
            let combined = reader
                .into_inner()
                .reunite(writer)
                .map_err(|_| anyhow!("Failed to reunite stream"))?;
            let (_reader, mut writer) = futures::io::AsyncReadExt::split(combined);
            writer.write_all(b"A4 LOGOUT\r\n").await?;
            writer.flush().await?;
        }

        println!("[IMAP] Fetched {} emails", emails.len());
        Ok(emails)
    }

    /// Wait for verification code email
    pub async fn wait_for_verification_code(
        &self,
        client_id: &str,
        refresh_token: &str,
        email: &str,
        timeout_seconds: u64,
    ) -> Result<String> {
        println!(
            "[IMAP] Starting to wait for verification code for email: {}",
            email
        );
        println!("[IMAP] Timeout: {} seconds", timeout_seconds);

        let access_token = Self::get_access_token(client_id, refresh_token).await?;

        let start_time = Utc::now();
        let timeout_duration = Duration::seconds(timeout_seconds as i64);

        let mut attempt = 0;
        loop {
            attempt += 1;
            let elapsed = Utc::now() - start_time;

            println!(
                "[IMAP] Attempt #{} (elapsed: {}s)",
                attempt,
                elapsed.num_seconds()
            );

            // Check if timeout
            if elapsed > timeout_duration {
                eprintln!("[IMAP] Timeout after {} seconds", elapsed.num_seconds());
                return Err(anyhow!("Timeout waiting for verification code"));
            }

            // Fetch recent emails from multiple mailboxes
            let mailboxes = vec!["INBOX", "Junk", "Spam"];
            let mut all_emails = Vec::new();

            for mailbox in &mailboxes {
                match self
                    .fetch_recent_emails(email, &access_token, mailbox, 20)
                    .await
                {
                    Ok(emails) => {
                        println!("[IMAP] Fetched {} emails from {}", emails.len(), mailbox);
                        all_emails.extend(emails);
                    }
                    Err(e) => {
                        // It's OK if some mailboxes don't exist (e.g., Spam might not exist)
                        println!(
                            "[IMAP] Could not fetch from {} (might not exist): {}",
                            mailbox, e
                        );
                    }
                }
            }

            if !all_emails.is_empty() {
                println!(
                    "[IMAP] Total fetched {} emails from all mailboxes",
                    all_emails.len()
                );

                // Print all email senders for debugging
                println!("[IMAP] Email list:");
                for (i, (from, _body)) in all_emails.iter().enumerate() {
                    println!("[IMAP]   #{}: From: {}", i + 1, from);
                }

                // Look for verification code in emails from kiro.dev
                for (from, body) in all_emails.iter().rev() {
                    // Check if from kiro.dev or AWS
                    if from.to_lowercase().contains("kiro.dev")
                        || from.to_lowercase().contains("kiro")
                        || from.to_lowercase().contains("no-reply")
                        || from.to_lowercase().contains("noreply")
                        || from.to_lowercase().contains("aws")
                        || from.to_lowercase().contains("signin.aws")
                    {
                        println!("[IMAP] Checking email from: {}", from);

                        // Safe substring for multi-byte characters (like Chinese)
                        let preview = body.chars().take(200).collect::<String>();
                        println!("[IMAP] Body preview (first 200 chars): {}", preview);

                        if let Some(code) = Self::extract_verification_code(body) {
                            println!("[IMAP] ✓ Found verification code: {}", code);
                            return Ok(code);
                        } else {
                            println!("[IMAP] No verification code found in this email");
                        }
                    }
                }

                println!("[IMAP] No verification code found in any email");
            } else {
                println!("[IMAP] No emails found in any mailbox");
            }

            println!("[IMAP] Waiting 5 seconds before retry...");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}
