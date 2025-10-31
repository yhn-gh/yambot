use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::auth;
use super::error::{Result, TwitchError};

const EVENTSUB_API_URL: &str = "https://api.twitch.tv/helix/eventsub/subscriptions";

/// EventSub subscription request
#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionRequest {
    #[serde(rename = "type")]
    pub subscription_type: String,
    pub version: String,
    pub condition: serde_json::Value,
    pub transport: Transport,
}

#[derive(Debug, Clone, Serialize)]
pub struct Transport {
    pub method: String,
    pub session_id: String,
}

/// EventSub subscription response
#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionResponse {
    #[allow(dead_code)]
    pub data: Vec<SubscriptionData>,
    #[allow(dead_code)]
    pub total: u32,
    #[allow(dead_code)]
    pub total_cost: u32,
    #[allow(dead_code)]
    pub max_total_cost: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionData {
    pub id: String,
    #[serde(rename = "type")]
    pub subscription_type: String,
    pub version: String,
    pub status: String,
    pub cost: u32,
    pub condition: serde_json::Value,
    pub created_at: String,
    pub transport: TransportData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransportData {
    #[allow(dead_code)]
    pub method: String,
    #[allow(dead_code)]
    pub session_id: String,
}

/// EventSub manager for creating and managing subscriptions
pub struct EventSubManager {
    client: reqwest::Client,
    access_token: Arc<RwLock<String>>,
    refresh_token: Arc<RwLock<String>>,
    token_refresh_tx: Option<mpsc::UnboundedSender<(String, String)>>,
}

impl EventSubManager {
    pub fn new(access_token: Arc<RwLock<String>>, refresh_token: Arc<RwLock<String>>) -> Self {
        Self {
            client: reqwest::Client::new(),
            access_token,
            refresh_token,
            token_refresh_tx: None,
        }
    }

    /// Set a channel to receive notifications when tokens are refreshed
    pub fn set_token_refresh_notifier(&mut self, tx: mpsc::UnboundedSender<(String, String)>) {
        self.token_refresh_tx = Some(tx);
    }

    /// Refresh the access token using the refresh token
    async fn refresh_token(&self) -> Result<()> {
        let current_refresh_token = self.refresh_token.read().await.clone();

        let token_response = auth::refresh_access_token(&current_refresh_token).await?;

        // Update both tokens
        let new_access_token = token_response.access_token.clone();
        let new_refresh_token = token_response.refresh_token.clone();

        {
            let mut access_token = self.access_token.write().await;
            *access_token = new_access_token.clone();
        }
        {
            let mut refresh_token = self.refresh_token.write().await;
            *refresh_token = new_refresh_token.clone();
        }

        // Notify listeners that tokens were refreshed
        if let Some(tx) = &self.token_refresh_tx {
            let _ = tx.send((new_access_token, new_refresh_token));
        }

        Ok(())
    }

    /// Create a new EventSub subscription
    async fn create_subscription(
        &self,
        request: SubscriptionRequest,
    ) -> Result<SubscriptionResponse> {
        let access_token = self.access_token.read().await;

        let response = self
            .client
            .post(EVENTSUB_API_URL)
            .header("Authorization", format!("Bearer {}", *access_token))
            .header("Client-Id", auth::CLIENT_ID)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();

            // Handle 401 by refreshing token and retrying
            if status.as_u16() == 401 {
                drop(access_token); // Release the lock before refreshing
                log::warn!("EventSub subscription got 401, refreshing token and retrying...");
                self.refresh_token().await?;
                return Box::pin(self.create_subscription(request)).await; // Retry with new token
            }

            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TwitchError::SubscriptionError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let subscription_response = response.json::<SubscriptionResponse>().await?;
        Ok(subscription_response)
    }

    /// Subscribe to channel chat messages
    pub async fn subscribe_to_chat_messages(
        &self,
        session_id: &str,
        broadcaster_user_id: &str,
        user_id: &str,
    ) -> Result<SubscriptionResponse> {
        let request = SubscriptionRequest {
            subscription_type: "channel.chat.message".to_string(),
            version: "1".to_string(),
            condition: json!({
                "broadcaster_user_id": broadcaster_user_id,
                "user_id": user_id
            }),
            transport: Transport {
                method: "websocket".to_string(),
                session_id: session_id.to_string(),
            },
        };

        self.create_subscription(request).await
    }

    /// Subscribe to message deletions
    pub async fn subscribe_to_message_delete(
        &self,
        session_id: &str,
        broadcaster_user_id: &str,
        user_id: &str,
    ) -> Result<SubscriptionResponse> {
        let request = SubscriptionRequest {
            subscription_type: "channel.chat.message_delete".to_string(),
            version: "1".to_string(),
            condition: json!({
                "broadcaster_user_id": broadcaster_user_id,
                "user_id": user_id
            }),
            transport: Transport {
                method: "websocket".to_string(),
                session_id: session_id.to_string(),
            },
        };

        self.create_subscription(request).await
    }

    /// Subscribe to user message clears (bans/timeouts)
    pub async fn subscribe_to_clear_user_messages(
        &self,
        session_id: &str,
        broadcaster_user_id: &str,
        user_id: &str,
    ) -> Result<SubscriptionResponse> {
        let request = SubscriptionRequest {
            subscription_type: "channel.chat.clear_user_messages".to_string(),
            version: "1".to_string(),
            condition: json!({
                "broadcaster_user_id": broadcaster_user_id,
                "user_id": user_id
            }),
            transport: Transport {
                method: "websocket".to_string(),
                session_id: session_id.to_string(),
            },
        };

        self.create_subscription(request).await
    }

    /// Subscribe to chat clear events
    pub async fn subscribe_to_chat_clear(
        &self,
        session_id: &str,
        broadcaster_user_id: &str,
        user_id: &str,
    ) -> Result<SubscriptionResponse> {
        let request = SubscriptionRequest {
            subscription_type: "channel.chat.clear".to_string(),
            version: "1".to_string(),
            condition: json!({
                "broadcaster_user_id": broadcaster_user_id,
                "user_id": user_id
            }),
            transport: Transport {
                method: "websocket".to_string(),
                session_id: session_id.to_string(),
            },
        };

        self.create_subscription(request).await
    }

    /// Subscribe to chat settings updates
    pub async fn subscribe_to_chat_settings_update(
        &self,
        session_id: &str,
        broadcaster_user_id: &str,
        user_id: &str,
    ) -> Result<SubscriptionResponse> {
        let request = SubscriptionRequest {
            subscription_type: "channel.chat_settings.update".to_string(),
            version: "1".to_string(),
            condition: json!({
                "broadcaster_user_id": broadcaster_user_id,
                "user_id": user_id
            }),
            transport: Transport {
                method: "websocket".to_string(),
                session_id: session_id.to_string(),
            },
        };

        self.create_subscription(request).await
    }

    /// Subscribe to channel ban events
    pub async fn subscribe_to_channel_ban(
        &self,
        session_id: &str,
        broadcaster_user_id: &str,
    ) -> Result<SubscriptionResponse> {
        let request = SubscriptionRequest {
            subscription_type: "channel.ban".to_string(),
            version: "1".to_string(),
            condition: json!({
                "broadcaster_user_id": broadcaster_user_id
            }),
            transport: Transport {
                method: "websocket".to_string(),
                session_id: session_id.to_string(),
            },
        };

        self.create_subscription(request).await
    }

    /// Subscribe to channel unban events
    pub async fn subscribe_to_channel_unban(
        &self,
        session_id: &str,
        broadcaster_user_id: &str,
    ) -> Result<SubscriptionResponse> {
        let request = SubscriptionRequest {
            subscription_type: "channel.unban".to_string(),
            version: "1".to_string(),
            condition: json!({
                "broadcaster_user_id": broadcaster_user_id
            }),
            transport: Transport {
                method: "websocket".to_string(),
                session_id: session_id.to_string(),
            },
        };

        self.create_subscription(request).await
    }

    /// Helper to get required scope for a subscription type
    fn get_required_scope(subscription_type: &str) -> &'static str {
        match subscription_type {
            "channel.chat.message" => "user:read:chat",
            "channel.chat.message_delete" => "user:read:chat",
            "channel.chat.clear_user_messages" => "user:read:chat",
            "channel.chat.clear" => "user:read:chat",
            "channel.chat_settings.update" => "user:read:chat",
            "channel.ban" => "channel:moderate or moderator:read:banned_users",
            "channel.unban" => "channel:moderate or moderator:read:banned_users",
            _ => "unknown scope",
        }
    }

    /// Subscribe to an event with error handling
    async fn subscribe_with_error_handling(
        &self,
        name: &str,
        subscription_type: &str,
        result: Result<SubscriptionResponse>,
        warnings: &mut Vec<String>,
    ) -> bool {
        match result {
            Ok(_) => {
                log::info!("✓ Subscribed to {} successfully", name);
                true
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("403") || error_msg.contains("Forbidden") || error_msg.contains("authorization") {
                    let warning = format!(
                        "Skipped '{}' - Missing OAuth scope: {}",
                        name,
                        Self::get_required_scope(subscription_type)
                    );
                    log::warn!("⚠ {}", warning);
                    warnings.push(warning);
                } else {
                    log::error!("✗ Failed to subscribe to {}: {}", name, e);
                }
                false
            }
        }
    }

    /// Subscribe to all chat events (continues on errors)
    /// Returns (success_count, failed_count, warnings)
    pub async fn subscribe_to_all_events(
        &self,
        session_id: &str,
        broadcaster_user_id: &str,
        user_id: &str,
    ) -> Result<(usize, usize, Vec<String>)> {
        log::info!("Creating EventSub subscriptions...");
        let mut success_count = 0;
        let mut failed_count = 0;
        let mut warnings = Vec::new();

        // Subscribe to all chat-related events (don't fail on errors)
        if self.subscribe_with_error_handling(
            "chat messages",
            "channel.chat.message",
            self.subscribe_to_chat_messages(session_id, broadcaster_user_id, user_id).await,
            &mut warnings,
        ).await {
            success_count += 1;
        } else {
            failed_count += 1;
        }

        if self.subscribe_with_error_handling(
            "message deletions",
            "channel.chat.message_delete",
            self.subscribe_to_message_delete(session_id, broadcaster_user_id, user_id).await,
            &mut warnings,
        ).await {
            success_count += 1;
        } else {
            failed_count += 1;
        }

        if self.subscribe_with_error_handling(
            "user message clears",
            "channel.chat.clear_user_messages",
            self.subscribe_to_clear_user_messages(session_id, broadcaster_user_id, user_id).await,
            &mut warnings,
        ).await {
            success_count += 1;
        } else {
            failed_count += 1;
        }

        if self.subscribe_with_error_handling(
            "chat clear",
            "channel.chat.clear",
            self.subscribe_to_chat_clear(session_id, broadcaster_user_id, user_id).await,
            &mut warnings,
        ).await {
            success_count += 1;
        } else {
            failed_count += 1;
        }

        if self.subscribe_with_error_handling(
            "chat settings updates",
            "channel.chat_settings.update",
            self.subscribe_to_chat_settings_update(session_id, broadcaster_user_id, user_id).await,
            &mut warnings,
        ).await {
            success_count += 1;
        } else {
            failed_count += 1;
        }

        if self.subscribe_with_error_handling(
            "channel bans",
            "channel.ban",
            self.subscribe_to_channel_ban(session_id, broadcaster_user_id).await,
            &mut warnings,
        ).await {
            success_count += 1;
        } else {
            failed_count += 1;
        }

        if self.subscribe_with_error_handling(
            "channel unbans",
            "channel.unban",
            self.subscribe_to_channel_unban(session_id, broadcaster_user_id).await,
            &mut warnings,
        ).await {
            success_count += 1;
        } else {
            failed_count += 1;
        }

        log::info!(
            "Subscriptions complete: {} succeeded, {} failed/skipped",
            success_count,
            failed_count
        );

        // Warn if ALL subscriptions failed but don't error out
        // This allows the bot to stay connected and show warnings to user
        if success_count == 0 {
            log::error!("All EventSub subscriptions failed - bot will not receive chat events!");
            warnings.push(
                "❌ All EventSub subscriptions failed! Bot will not receive chat messages or events.".to_string()
            );
            warnings.push(
                "Required OAuth scopes: user:read:chat, user:write:chat".to_string()
            );
            warnings.push(
                "Please re-authorize with proper scopes in Settings.".to_string()
            );
        }

        Ok((success_count, failed_count, warnings))
    }
}
