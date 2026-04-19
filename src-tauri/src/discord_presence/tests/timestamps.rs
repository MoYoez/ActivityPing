use super::{super::*, fixtures::sample_media};
use chrono::Utc;

#[test]
fn paused_media_uses_future_timestamps_when_visible() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Music,
        report_stopped_media: true,
        ..ClientConfig::default()
    };
    let mut media = sample_media();
    media.is_playing = false;
    media.position_ms = Some(30_000);
    media.duration_ms = Some(180_000);

    let (started_at, ended_at) = build_media_timestamps(&config, &media);
    let started_at = started_at.expect("start timestamp");
    let ended_at = ended_at.expect("end timestamp");

    assert!(started_at > Utc::now().timestamp_millis());
    assert_eq!(ended_at - started_at, 180_000);
}

#[test]
fn playing_timestamp_update_skips_small_end_drift() {
    let payload = DiscordPresencePayload {
        activity_name: None,
        details: "Track Name".into(),
        state: Some("Artist Name".into()),
        status_display_type: None,
        started_at_millis: Some(900_000),
        ended_at_millis: Some(1_000_050),
        media_duration_ms: Some(180_000),
        media_position_ms: Some(30_000),
        media_is_playing: true,
        summary: "Track Name".into(),
        signature: "Track Name".into(),
        artwork: None,
        icon: None,
        buttons: Vec::new(),
        party: None,
        secrets: None,
    };

    assert!(should_skip_timestamp_update(&payload, Some(1_000_000)));
    assert!(!should_skip_timestamp_update(&payload, Some(999_800)));
}
