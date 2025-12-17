// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::{Arc, Mutex};

use lingua::LanguageDetector;
use llama_cpp_2::{llama_backend::LlamaBackend, model::LlamaModel};
use rust_bert::pipelines::translation::TranslationModel;
use serde::{Deserialize, Serialize};
use tauri::Manager;

mod model;

struct RefiningModelState {
    backend: LlamaBackend,
    llm: LlamaModel,
}

struct TranslationModelState {
    detector: LanguageDetector,
    translation_model: Arc<Mutex<TranslationModel>>,
    llm: Arc<RefiningModelState>,
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![translate])
        .setup(move |app| {
            let app_handle = app.handle();
            let translation_model =
                model::initialize_translation_model_from_app_handle(&app_handle)
                    .expect("failed to load m2m100 model!");

            let llama_backend =
                model::initialize_llama_backend().expect("Failed to load llamacpp backend!");

            let llm = model::initialize_llm_from_app_handle(&app_handle, &llama_backend)
                .expect("failed to load qwen3 model!");

            app_handle.manage(TranslationModelState {
                detector: model::initialize_lingua(),
                translation_model: Arc::new(Mutex::new(translation_model)),
                llm: Arc::new(RefiningModelState {
                    backend: llama_backend,
                    llm: llm,
                }),
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
    let mut result =
        model::process_message(&text, &state.detector, &state.translation_model, &state.llm)
            .map_err(|e| format!("Translation Error: {}", e))?;

    if result.language != "English" {
        result.translation = model::refine_with_qwen(
            &state.llm.backend,
            &state.llm.llm,
            &result.language,
            &result.translation,
        )
        .map_err(|e| format!("LLM Error: {}", e))?;
    }
    println!("{:?}", result);
    Ok(result)
}
