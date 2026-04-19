use super::{super::*, fixtures::*};

#[test]
fn music_mode_uses_title_then_artist() {
    let config = base_config();
    let media = sample_media();

    let resolved = build_music_discord_text(&config, &media, false);

    assert_eq!(
        resolved,
        Some(("Track Name".to_string(), Some("Artist Name".to_string())))
    );
}

#[test]
fn music_mode_drops_state_when_artist_missing() {
    let config = base_config();
    let mut media = sample_media();
    media.artist.clear();

    let resolved = build_music_discord_text(&config, &media, false);

    assert_eq!(resolved, Some(("Track Name".to_string(), None)));
}

#[test]
fn music_mode_can_keep_paused_media_visible() {
    let mut config = base_config();
    config.report_stopped_media = true;
    let mut media = sample_media();
    media.is_playing = false;

    let resolved = build_music_discord_text(&config, &media, false);

    assert_eq!(
        resolved,
        Some(("Track Name".to_string(), Some("Artist Name".to_string())))
    );
}

#[test]
fn music_mode_hides_paused_media_by_default() {
    let config = base_config();
    let mut media = sample_media();
    media.is_playing = false;

    let resolved = build_music_discord_text(&config, &media, false);

    assert_eq!(resolved, None);
}
