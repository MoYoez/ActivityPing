use super::super::*;

#[test]
fn normalize_client_config_migrates_legacy_artwork_toggle() {
    let mut config = ClientConfig {
        legacy_discord_use_media_artwork: true,
        ..ClientConfig::default()
    };

    normalize_client_config(&mut config);

    assert!(config.discord_use_app_artwork);
    assert!(config.discord_use_music_artwork);
    assert!(!config.legacy_discord_use_media_artwork);
}
