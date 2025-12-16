// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::{Arc, Mutex};

use lingua::LanguageDetector;
use rust_bert::pipelines::translation::TranslationModel;
use serde::{Deserialize, Serialize};
use tauri::Manager;

mod model;

struct TranslationModelState {
    detector: LanguageDetector,
    translation_model: Arc<Mutex<TranslationModel>>,
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![translate])
        .setup(move |app| {
            let app_handle = app.handle();
            let translation_model =
                model::initialize_translation_model_from_app_handle(&app_handle)
                    .expect("failed to load translation model!");
            app_handle.manage(TranslationModelState {
                detector: model::initialize_lingua(),
                translation_model: Arc::new(Mutex::new(translation_model)),
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Serialize, Deserialize, Debug)]
struct TranslationResponse {
    language: String,
    translation: String,
}

#[tauri::command]
async fn translate(
    text: String,
    state: tauri::State<'_, TranslationModelState>,
) -> Result<TranslationResponse, String> {
    let model_lock = state
        .translation_model
        .lock()
        .map_err(|e| format!("Failed to lock translation model: {}", e))?;
    let result = model::process_message(&text, &state.detector, &model_lock);
    println!("{:?}", result);
    result
}
