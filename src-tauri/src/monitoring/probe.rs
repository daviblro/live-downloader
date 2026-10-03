pub(super) const MAX_CONCURRENT_PROBES: usize = 12;

/// yt-dlp output template printed by the live check. `live_status` lets the
/// probe tell a running broadcast apart from a VOD or replay, which yt-dlp can
/// otherwise "simulate" successfully.
pub(super) const LIVE_STATUS_TEMPLATE: &str = "%(live_status)s";

#[derive(Debug)]
pub(super) enum ProbeOutcome {
    Live,
    Offline,
    Failed(String),
}

pub(super) fn classify_probe_output(success: bool, stdout: &[u8], stderr: &[u8]) -> ProbeOutcome {
    if success {
        return classify_live_status(stdout);
    }
    const MAX_DETAIL_BYTES: usize = 2_000;
    let combined = [stderr, stdout].concat();
    let start = combined.len().saturating_sub(MAX_DETAIL_BYTES);
    let detail = String::from_utf8_lossy(&combined[start..])
        .trim()
        .to_owned();
    let normalized = detail.to_ascii_lowercase();
    let offline = [
        "not currently live",
        "is offline",
        "no live formats",
        "premieres in",
        "this live event will begin",
    ]
    .iter()
    .any(|message| normalized.contains(message));
    if offline {
        ProbeOutcome::Offline
    } else if detail.is_empty() {
        ProbeOutcome::Failed("yt-dlp could not check the stream (no diagnostic output)".to_owned())
    } else {
        ProbeOutcome::Failed(format!("yt-dlp could not check the stream: {detail}"))
    }
}

fn classify_live_status(stdout: &[u8]) -> ProbeOutcome {
    let output = String::from_utf8_lossy(stdout);
    let status = output
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match status.as_str() {
        // Finished broadcasts, replays, regular videos and scheduled streams
        // must not be downloaded as if they were live.
        "was_live" | "not_live" | "post_live" | "is_upcoming" => ProbeOutcome::Offline,
        // "is_live", plus direct stream URLs whose extractor does not report a
        // live status ("NA"), keep the previous behaviour and are recorded.
        _ => ProbeOutcome::Live,
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_probe_output, ProbeOutcome};

    #[test]
    fn distinguishes_offline_streams_from_probe_failures() {
        assert!(matches!(
            classify_probe_output(false, b"", b"ERROR: stream is not currently live"),
            ProbeOutcome::Offline
        ));
        assert!(
            matches!(classify_probe_output(false, b"", b"ERROR: unable to resolve host"), ProbeOutcome::Failed(message) if message.contains("resolve host"))
        );
        assert!(matches!(
            classify_probe_output(true, b"is_live\n", b""),
            ProbeOutcome::Live
        ));
    }

    #[test]
    fn does_not_treat_finished_broadcasts_or_videos_as_live() {
        for status in ["was_live", "not_live", "post_live", "is_upcoming"] {
            assert!(
                matches!(
                    classify_probe_output(true, format!("{status}\r\n").as_bytes(), b""),
                    ProbeOutcome::Offline
                ),
                "{status} should be offline"
            );
        }
    }

    #[test]
    fn records_direct_streams_without_a_reported_live_status() {
        assert!(matches!(
            classify_probe_output(true, b"NA\n", b""),
            ProbeOutcome::Live
        ));
        assert!(matches!(
            classify_probe_output(true, b"", b""),
            ProbeOutcome::Live
        ));
    }
}
