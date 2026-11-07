mod converter;
mod settings;

use std::path::PathBuf;
use tauri::Manager;

fn encoder_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .resolve("binaries/cwebp.exe", tauri::path::BaseDirectory::Resource)
        .map_err(|error| format!("Cannot locate bundled encoder: {error}"))
}

#[tauri::command]
async fn encoder_version(app: tauri::AppHandle) -> Result<String, String> {
    let binary = encoder_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || converter::version(&binary))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn inspect_files(paths: Vec<String>) -> Result<Vec<converter::InputFile>, String> {
    if paths.len() > 1000 {
        return Err("Add up to 1,000 files at a time.".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        paths.into_iter().map(converter::inspect).collect()
    })
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn validate_settings(settings: settings::EncoderSettings) -> Result<(), String> {
    settings.validate()
}

#[tauri::command]
async fn convert_image(
    app: tauri::AppHandle,
    input_path: String,
    output_directory: Option<String>,
    settings: settings::EncoderSettings,
) -> Result<converter::ConversionResult, String> {
    let binary = encoder_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        converter::convert(
            &binary,
            &PathBuf::from(input_path),
            output_directory.as_deref().map(std::path::Path::new),
            &settings,
        )
    })
    .await
    .map_err(|error| format!("Conversion worker failed: {error}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            encoder_version,
            inspect_files,
            validate_settings,
            convert_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running Anything to WebP");
}
