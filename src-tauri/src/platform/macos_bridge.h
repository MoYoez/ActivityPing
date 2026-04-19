#pragma once

#include <stdbool.h>

char *waken_frontmost_app_name(void);
char *waken_frontmost_app_bundle_identifier(void);
char *waken_frontmost_window_title(void);
char *waken_media_now_playing_json(void);
char *waken_bundle_icon_png_base64(const char *bundle_identifier, int target_size);
char *waken_bundle_display_name(const char *bundle_identifier);
bool waken_accessibility_is_trusted(void);
bool waken_request_accessibility_permission(void);
void waken_string_free(char *value);
