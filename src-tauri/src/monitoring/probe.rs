pub(super) const MAX_CONCURRENT_PROBES: usize = 12;

#[derive(Debug)]
pub(super) enum ProbeOutcome {
    Live,
    Offline,
    Failed(String),
}

pub(super) fn classify_probe_output(success: bool, stdout: &[u8], stderr: &[u8]) -> ProbeOutcome {
    if success {
        return ProbeOutcome::Live;
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
            classify_probe_output(true, b"", b""),
            ProbeOutcome::Live
        ));
    }
}
