use keyring::Entry;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use url::Url;

use crate::domain::TargetMetadata;

const CREDENTIAL_SERVICE: &str = "Live Downloader";
const CREDENTIAL_USER: &str = "twitch-oauth";
const DEVICE_URL: &str = "https://id.twitch.tv/oauth2/device";
const TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";
const VALIDATE_URL: &str = "https://id.twitch.tv/oauth2/validate";
const USERS_URL: &str = "https://api.twitch.tv/helix/users";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TwitchStatus {
    pub available: bool,
    pub connected: bool,
    pub login: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct TwitchDeviceAuthorization {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TwitchConnectResult {
    pub connected: bool,
    pub login: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct StoredToken {
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Deserialize)]
struct TwitchErrorResponse {
    message: String,
}

#[derive(Debug, Deserialize)]
struct ValidationResponse {
    login: String,
}

#[derive(Debug, Deserialize)]
struct UsersResponse {
    data: Vec<TwitchUser>,
}

#[derive(Debug, Deserialize)]
struct TwitchUser {
    id: String,
    profile_image_url: String,
}

pub struct TwitchService {
    client_id: Option<String>,
    http: Client,
    refresh_lock: Mutex<()>,
}

impl TwitchService {
    pub fn new(client_id: Option<&str>) -> Self {
        Self {
            client_id: client_id
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
            http: Client::new(),
            refresh_lock: Mutex::new(()),
        }
    }

    pub async fn status(&self) -> Result<TwitchStatus, String> {
        if self.client_id.is_none() {
            return Ok(TwitchStatus {
                available: false,
                connected: false,
                login: None,
            });
        }
        let Some(token) = self.load_token()? else {
            return Ok(TwitchStatus {
                available: true,
                connected: false,
                login: None,
            });
        };
        let response = self.validate(&token.access_token).await?;
        if response.status() == StatusCode::UNAUTHORIZED {
            let refreshed = match self.refresh(&token.access_token).await {
                Ok(token) => token,
                Err(_) => {
                    self.delete_token()?;
                    return Ok(TwitchStatus {
                        available: true,
                        connected: false,
                        login: None,
                    });
                }
            };
            return self.status_for_token(&refreshed.access_token).await;
        }
        self.status_from_response(response).await
    }

    pub async fn start_connect(&self) -> Result<TwitchDeviceAuthorization, String> {
        let client_id = self.client_id()?;
        let response = self
            .http
            .post(DEVICE_URL)
            .form(&[("client_id", client_id), ("scopes", "")])
            .send()
            .await
            .map_err(network_error)?;
        decode_response(response).await
    }

    pub async fn poll_connect(&self, device_code: &str) -> Result<TwitchConnectResult, String> {
        let client_id = self.client_id()?;
        let response = self
            .http
            .post(TOKEN_URL)
            .form(&[
                ("client_id", client_id),
                ("scopes", ""),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await
            .map_err(network_error)?;
        if response.status() == StatusCode::BAD_REQUEST {
            let error = response
                .json::<TwitchErrorResponse>()
                .await
                .map_err(network_error)?;
            if error.message == "authorization_pending" {
                return Ok(TwitchConnectResult {
                    connected: false,
                    login: None,
                });
            }
            return Err(error.message);
        }
        let token: TokenResponse = decode_response(response).await?;
        let stored = StoredToken {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
        };
        self.save_token(&stored)?;
        let status = self.status_for_token(&stored.access_token).await?;
        Ok(TwitchConnectResult {
            connected: true,
            login: status.login,
        })
    }

    pub fn disconnect(&self) -> Result<(), String> {
        self.delete_token()
    }

    pub async fn metadata_for_url(&self, value: &str) -> Result<Option<TargetMetadata>, String> {
        let Some(login) = twitch_login(value) else {
            return Ok(None);
        };
        let Some(token) = self.load_token()? else {
            return Ok(None);
        };
        let mut response = self.user(&login, &token.access_token).await?;
        if response.status() == StatusCode::UNAUTHORIZED {
            let refreshed = self.refresh(&token.access_token).await?;
            response = self.user(&login, &refreshed.access_token).await?;
        }
        let users: UsersResponse = decode_response(response).await?;
        Ok(users.data.into_iter().next().map(|user| TargetMetadata {
            provider_user_id: user.id,
            avatar_url: user.profile_image_url,
        }))
    }

    async fn status_for_token(&self, access_token: &str) -> Result<TwitchStatus, String> {
        let response = self.validate(access_token).await?;
        self.status_from_response(response).await
    }

    async fn status_from_response(
        &self,
        response: reqwest::Response,
    ) -> Result<TwitchStatus, String> {
        if !response.status().is_success() {
            return Err(response_error(response).await);
        }
        let validation = response
            .json::<ValidationResponse>()
            .await
            .map_err(network_error)?;
        Ok(TwitchStatus {
            available: true,
            connected: true,
            login: Some(validation.login),
        })
    }

    async fn validate(&self, access_token: &str) -> Result<reqwest::Response, String> {
        self.http
            .get(VALIDATE_URL)
            .header("Authorization", format!("OAuth {access_token}"))
            .send()
            .await
            .map_err(network_error)
    }

    async fn user(&self, login: &str, access_token: &str) -> Result<reqwest::Response, String> {
        self.http
            .get(USERS_URL)
            .query(&[("login", login)])
            .bearer_auth(access_token)
            .header("Client-Id", self.client_id()?)
            .send()
            .await
            .map_err(network_error)
    }

    async fn refresh(&self, failed_access_token: &str) -> Result<StoredToken, String> {
        let _guard = self.refresh_lock.lock().await;
        let current = self
            .load_token()?
            .ok_or_else(|| "Twitch is not connected.".to_owned())?;
        if current.access_token != failed_access_token {
            return Ok(current);
        }
        let response = self
            .http
            .post(TOKEN_URL)
            .form(&[
                ("client_id", self.client_id()?),
                ("grant_type", "refresh_token"),
                ("refresh_token", current.refresh_token.as_str()),
            ])
            .send()
            .await
            .map_err(network_error)?;
        let token: TokenResponse = decode_response(response).await?;
        let stored = StoredToken {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
        };
        self.save_token(&stored)?;
        Ok(stored)
    }

    fn client_id(&self) -> Result<&str, String> {
        self.client_id.as_deref().ok_or_else(|| {
            "Twitch integration is not configured in this build. Set TWITCH_CLIENT_ID when building the app."
                .to_owned()
        })
    }

    fn credential() -> Result<Entry, String> {
        Entry::new(CREDENTIAL_SERVICE, CREDENTIAL_USER).map_err(|error| error.to_string())
    }

    fn load_token(&self) -> Result<Option<StoredToken>, String> {
        match Self::credential()?.get_password() {
            Ok(value) => serde_json::from_str(&value)
                .map(Some)
                .map_err(|error| error.to_string()),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }

    fn save_token(&self, token: &StoredToken) -> Result<(), String> {
        let value = serde_json::to_string(token).map_err(|error| error.to_string())?;
        Self::credential()?
            .set_password(&value)
            .map_err(|error| error.to_string())
    }

    fn delete_token(&self) -> Result<(), String> {
        match Self::credential()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}

async fn decode_response<T: for<'de> Deserialize<'de>>(
    response: reqwest::Response,
) -> Result<T, String> {
    if response.status().is_success() {
        response.json::<T>().await.map_err(network_error)
    } else {
        Err(response_error(response).await)
    }
}

async fn response_error(response: reqwest::Response) -> String {
    let status = response.status();
    response
        .json::<TwitchErrorResponse>()
        .await
        .map(|error| error.message)
        .unwrap_or_else(|_| format!("Twitch returned {status}."))
}

fn network_error(error: reqwest::Error) -> String {
    format!("Could not reach Twitch: {error}")
}

fn twitch_login(value: &str) -> Option<String> {
    let url = Url::parse(value).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    if !matches!(host.as_str(), "twitch.tv" | "www.twitch.tv" | "m.twitch.tv") {
        return None;
    }
    let login = url.path_segments()?.find(|segment| !segment.is_empty())?;
    let normalized = login.to_ascii_lowercase();
    if matches!(
        normalized.as_str(),
        "directory" | "downloads" | "jobs" | "p" | "settings" | "subscriptions" | "videos"
    ) || !normalized
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return None;
    }
    Some(normalized)
}

#[cfg(test)]
mod tests {
    use super::{twitch_login, TwitchDeviceAuthorization};

    #[test]
    fn twitch_login_extracts_channel_from_supported_url() {
        assert_eq!(
            twitch_login("https://www.twitch.tv/TwitchDev"),
            Some("twitchdev".to_owned())
        );
    }

    #[test]
    fn twitch_login_rejects_non_twitch_url() {
        assert_eq!(twitch_login("https://example.com/twitchdev"), None);
    }

    #[test]
    fn twitch_login_rejects_reserved_twitch_route() {
        assert_eq!(twitch_login("https://www.twitch.tv/directory"), None);
    }

    #[test]
    fn device_authorization_deserializes_twitch_response_fields() {
        let authorization: TwitchDeviceAuthorization = serde_json::from_str(
            r#"{"device_code":"device","user_code":"ABC123","verification_uri":"https://www.twitch.tv/activate","expires_in":1800,"interval":5}"#,
        )
        .expect("device response should deserialize");

        assert_eq!(authorization.user_code, "ABC123");
    }
}
