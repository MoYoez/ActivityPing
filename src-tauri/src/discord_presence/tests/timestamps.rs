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

#[test]
fn stalled_playing_media_falls_back_to_paused_timestamps() {
    let mut playback_state = PlaybackProgressState::default();
    let mut first_payload = DiscordPresencePayload {
        activity_name: None,
        details: "Track Name".into(),
        state: Some("Artist Name".into()),
        status_display_type: None,
        started_at_millis: Some(900_000),
        ended_at_millis: Some(1_050_000),
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

    downgrade_stalled_playback_to_paused(&mut first_payload, &mut playback_state, 2_500);
    assert!(first_payload.media_is_playing);

    let mut second_payload = first_payload.clone();
    second_payload.media_is_playing = true;
    second_payload.started_at_millis = Some(905_000);
    second_payload.ended_at_millis = Some(1_055_000);

    downgrade_stalled_playback_to_paused(&mut second_payload, &mut playback_state, 2_500);

    assert!(!second_payload.media_is_playing);
    assert!(second_payload.started_at_millis.is_some());
    assert!(second_payload.ended_at_millis.is_some());
    assert_eq!(
        second_payload
            .ended_at_millis
            .expect("paused end timestamp")
            - second_payload
                .started_at_millis
                .expect("paused start timestamp"),
        180_000
    );
}

#[test]
fn advancing_playback_keeps_playing_timestamps() {
    let mut playback_state = PlaybackProgressState::default();
    let mut first_payload = DiscordPresencePayload {
        activity_name: None,
        details: "Track Name".into(),
        state: Some("Artist Name".into()),
        status_display_type: None,
        started_at_millis: Some(900_000),
        ended_at_millis: Some(1_050_000),
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

    downgrade_stalled_playback_to_paused(&mut first_payload, &mut playback_state, 2_500);

    let original_started_at = 905_000;
    let original_ended_at = 1_055_000;
    let mut second_payload = first_payload.clone();
    second_payload.media_is_playing = true;
    second_payload.started_at_millis = Some(original_started_at);
    second_payload.ended_at_millis = Some(original_ended_at);
    second_payload.media_position_ms = Some(35_000);

    downgrade_stalled_playback_to_paused(&mut second_payload, &mut playback_state, 2_500);

    assert!(second_payload.media_is_playing);
    assert_eq!(second_payload.started_at_millis, Some(original_started_at));
    assert_eq!(second_payload.ended_at_millis, Some(original_ended_at));
}
