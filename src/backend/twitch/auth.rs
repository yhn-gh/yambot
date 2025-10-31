use super::error::{Result, TwitchError};
use serde::{Deserialize, Serialize};

/// Hardcoded client credentials - NOT exposed to users
pub const CLIENT_ID: &str = "";
const CLIENT_SECRET: &str = "";
const TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";

/// Response from the token refresh endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u32,
    pub scope: Vec<String>,
    pub token_type: String,
}

/// Refresh the access token using a refresh token
///
/// # Arguments
/// * `refresh_token` - The refresh token to use for getting a new access token
///
/// # Returns
/// A `TokenResponse` containing the new access token and refresh token
pub async fn refresh_access_token(refresh_token: &str) -> Result<TokenResponse> {
    let client = reqwest::Client::new();

    let params = [
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];

    let response = client.post(TOKEN_URL).form(&params).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(TwitchError::AuthError(format!(
            "Token refresh failed: HTTP {} - {}",
            status, error_text
        )));
    }

    let token_response = response.json::<TokenResponse>().await?;

    Ok(token_response)
}

/// Validate the current access token
///
/// # Arguments
/// * `access_token` - The access token to validate
///
/// # Returns
/// `true` if the token is valid, `false` otherwise
pub async fn validate_token(access_token: &str) -> Result<bool> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://id.twitch.tv/oauth2/validate")
        .header("Authorization", format!("OAuth {}", access_token))
        .send()
        .await?;

    Ok(response.status().is_success())
}
