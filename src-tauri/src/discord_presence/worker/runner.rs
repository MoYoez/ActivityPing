use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use chrono::Utc;
use discord_rich_presence::DiscordIpcClient;

use crate::{
    artwork_server::ArtworkPublisher, backend_locale::BackendLocale, models::ClientConfig,
};

use super::{
    super::{
        client::{apply_discord_presence, clear_discord_presence},
        state::{
            mark_stopped, set_discord_error, update_presence_heartbeat, update_presence_snapshot,
            DiscordPresenceInner,
        },
        timestamps::should_skip_timestamp_update,
        DEFAULT_SYNC_INTERVAL,
    },
    capture::capture_local_presence,
    lifecycle::{error_backoff, sleep_with_stop},
};

pub(super) fn run_discord_presence_loop(
    state: Arc<Mutex<DiscordPresenceInner>>,
    config: ClientConfig,
    stop_flag: Arc<AtomicBool>,
    run_id: u64,
    artwork_publisher: Option<ArtworkPublisher>,
    locale: BackendLocale,
) {
    let mut discord_client: Option<DiscordIpcClient> = None;
    let sync_interval = Duration::from_millis(
        config
            .poll_interval_ms
            .max(DEFAULT_SYNC_INTERVAL.as_millis() as u64),
    );
    let mut consecutive_errors = 0u32;
    let mut last_signature = String::new();
    let mut last_publish_signature = String::new();
    let mut last_sent_end_timestamp: Option<i64> = None;
    let mut activity_started_at = Some(Utc::now().timestamp_millis());

    while !stop_flag.load(Ordering::SeqCst) {
        match capture_local_presence(&config) {
            Ok(Some(mut payload)) => {
                let signature_changed = payload.signature != last_signature;
                if signature_changed {
                    last_signature = payload.signature.clone();
                    activity_started_at = Some(Utc::now().timestamp_millis());
                }
                if payload.started_at_millis.is_none() && payload.ended_at_millis.is_none() {
                    payload.started_at_millis = activity_started_at;
                }

                let publish_signature = payload.publish_signature();
                if publish_signature != last_publish_signature {
                    last_publish_signature = publish_signature;
                    last_sent_end_timestamp = None;
                }

                if should_skip_timestamp_update(&payload, last_sent_end_timestamp) {
                    update_presence_heartbeat(&state, true, None, run_id);
                    consecutive_errors = 0;
                    sleep_with_stop(sync_interval, &stop_flag);
                    continue;
                }

                match apply_discord_presence(
                    &mut discord_client,
                    &config,
                    &config.discord_application_id,
                    &payload,
                    artwork_publisher.as_ref(),
                    locale,
                ) {
                    Ok(debug_payload) => {
                        update_presence_snapshot(
                            &state,
                            true,
                            None,
                            Some(payload.summary),
                            Some(debug_payload),
                            run_id,
                        );
                        last_sent_end_timestamp =
                            if payload.media_is_playing && payload.ended_at_millis.is_some() {
                                payload.ended_at_millis
                            } else {
                                None
                            };
                        consecutive_errors = 0;
                        sleep_with_stop(sync_interval, &stop_flag);
                    }
                    Err(error) => {
                        discord_client = None;
                        last_sent_end_timestamp = None;
                        consecutive_errors = consecutive_errors.saturating_add(1);
                        set_discord_error(&state, Some(error), false, run_id);
                        sleep_with_stop(error_backoff(consecutive_errors), &stop_flag);
                    }
                }
            }
            Ok(None) => {
                let _ = clear_discord_presence(
                    &mut discord_client,
                    &config.discord_application_id,
                    locale,
                );
                update_presence_snapshot(&state, true, None, None, None, run_id);
                last_signature.clear();
                last_publish_signature.clear();
                last_sent_end_timestamp = None;
                consecutive_errors = 0;
                sleep_with_stop(sync_interval, &stop_flag);
            }
            Err(error) => {
                discord_client = None;
                last_sent_end_timestamp = None;
                consecutive_errors = consecutive_errors.saturating_add(1);
                set_discord_error(&state, Some(error), false, run_id);
                sleep_with_stop(error_backoff(consecutive_errors), &stop_flag);
            }
        }
    }

    let _ = clear_discord_presence(&mut discord_client, &config.discord_application_id, locale);
    mark_stopped(&state, run_id);
}
