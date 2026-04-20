use tauri::AppHandle;

use crate::{
    custom_assets,
    models::{ApiResult, DiscordCustomAsset},
};

pub fn import_discord_custom_asset(
    app: AppHandle,
    name: String,
    file_name: String,
    content_type: String,
    base64_data: String,
) -> Result<ApiResult<Vec<DiscordCustomAsset>>, String> {
    match custom_assets::import_discord_custom_asset(
        &app,
        &name,
        &file_name,
        &content_type,
        &base64_data,
    ) {
        Ok(assets) => Ok(ApiResult::success(200, assets)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            None::<String>,
            error,
            None,
            None,
        )),
    }
}

pub fn delete_discord_custom_asset(
    app: AppHandle,
    asset_id: String,
) -> Result<ApiResult<Vec<DiscordCustomAsset>>, String> {
    match custom_assets::delete_discord_custom_asset(&app, &asset_id) {
        Ok(assets) => Ok(ApiResult::success(200, assets)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            None::<String>,
            error,
            None,
            None,
        )),
    }
}

pub fn get_discord_custom_asset_preview(
    app: AppHandle,
    asset_id: String,
) -> Result<ApiResult<String>, String> {
    match custom_assets::get_discord_custom_asset_preview(&app, &asset_id) {
        Ok(preview) => Ok(ApiResult::success(200, preview)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            None::<String>,
            error,
            None,
            None,
        )),
    }
}
