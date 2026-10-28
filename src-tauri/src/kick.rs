//! Kick (kick.com) support. Live checks and recordings run through yt-dlp's
//! Kick extractor like any other source; this module validates Kick URLs up
//! front and turns Kick-specific yt-dlp failures into actionable messages.

use url::Url;

/// First path segments on kick.com that are site pages, not channels.
const RESERVED_ROUTES: &[&str] = &[
    "auth",
    "browse",
    "categories",
    "category",
    "dashboard",
    "following",
    "search",
    "settings",
    "subscriptions",
    "video",
];

pub fn is_kick_url(value: &str) -> bool {
    Url::parse(value)
        .ok()
        .and_then(|url| url.host_str().map(str::to_ascii_lowercase))
        .is_some_and(|host| matches!(host.as_str(), "kick.com" | "www.kick.com"))
}

/// Returns the lowercase channel slug of a Kick channel URL, or `None` when
/// the URL is not on Kick or points at a site page instead of a channel.
pub fn kick_channel(value: &str) -> Option<String> {
    if !is_kick_url(value) {
        return None;
    }
    let url = Url::parse(value).ok()?;
    let channel = url.path_segments()?.find(|segment| !segment.is_empty())?;
    let normalized = channel.to_ascii_lowercase();
    if RESERVED_ROUTES.contains(&normalized.as_str())
        || !normalized
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return None;
    }
    Some(normalized)
}

/// Rejects Kick URLs that cannot be monitored as a channel. Non-Kick URLs are
/// accepted unchanged.
pub fn validate_target_url(value: &str) -> Result<(), String> {
    if is_kick_url(value) && kick_channel(value).is_none() {
        return Err("Enter a Kick channel URL, such as https://kick.com/channel.".to_owned());
    }
    Ok(())
}

/// Adds a hint to a failed live check of a Kick URL when the cause is known.
pub fn explain_probe_failure(url: &str, message: String) -> String {
    if !is_kick_url(url) {
        return message;
    }
    let normalized = message.to_ascii_lowercase();
    if normalized.contains("http error 404") {
        format!("{message}\nThis Kick channel was not found. Check the channel name in the URL.")
    } else if normalized.contains("http error 403") || normalized.contains("impersonat") {
        format!(
            "{message}\nKick blocked the request. Kick requires a yt-dlp build with browser impersonation (curl_cffi). The bundled yt-dlp includes it; if you configured an external yt-dlp, install it with curl_cffi support or clear the external path in Settings."
        )
    } else {
        message
    }
}

#[cfg(test)]
mod tests {
    use super::{explain_probe_failure, kick_channel, validate_target_url};

    #[test]
    fn extracts_channel_from_kick_urls() {
        assert_eq!(kick_channel("https://kick.com/xQc"), Some("xqc".to_owned()));
        assert_eq!(
            kick_channel("https://www.kick.com/a-log_burner?ref=home"),
            Some("a-log_burner".to_owned())
        );
        assert_eq!(
            kick_channel("https://kick.com/xqc/videos/01a0251f-ef48-702d-9289-d66922da0e84"),
            Some("xqc".to_owned())
        );
    }

    #[test]
    fn rejects_kick_site_pages_and_other_hosts() {
        assert_eq!(kick_channel("https://kick.com/"), None);
        assert_eq!(
            kick_channel("https://kick.com/categories/just-chatting"),
            None
        );
        assert_eq!(kick_channel("https://example.com/xqc"), None);
        assert_eq!(kick_channel("https://notkick.com/xqc"), None);
    }

    #[test]
    fn validation_only_applies_to_kick_urls() {
        assert!(validate_target_url("https://kick.com/xqc").is_ok());
        assert!(validate_target_url("https://www.twitch.tv/twitchdev").is_ok());
        assert!(validate_target_url("https://kick.com/browse").is_err());
        assert!(validate_target_url("https://kick.com").is_err());
    }

    #[test]
    fn explains_blocked_and_missing_kick_channels() {
        let blocked = explain_probe_failure(
            "https://kick.com/xqc",
            "yt-dlp could not check the stream: ERROR: [kick:live] xqc: HTTP Error 403: Forbidden"
                .to_owned(),
        );
        assert!(blocked.contains("browser impersonation"));

        let missing = explain_probe_failure(
            "https://kick.com/nobody",
            "yt-dlp could not check the stream: ERROR: HTTP Error 404: Not Found".to_owned(),
        );
        assert!(missing.contains("not found"));

        let other = "yt-dlp could not check the stream: HTTP Error 403".to_owned();
        assert_eq!(
            explain_probe_failure("https://example.com/live", other.clone()),
            other
        );
    }
}
