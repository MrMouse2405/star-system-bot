use anyhow::Result;
use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};
use rust_bert::m2m_100::M2M100SourceLanguages;
use rust_bert::pipelines::common::ModelResource;
use rust_bert::pipelines::translation::{
    Language as BertLang, TranslationConfig, TranslationModel,
};
use rust_bert::resources::LocalResource;
use rust_bert::RustBertError;
use std::path::PathBuf;
use std::string::ToString;
use tauri::path::BaseDirectory;
use tauri::Manager;
use tch::Device;

use crate::TranslationResponse;

pub fn initialize_lingua() -> LanguageDetector {
    // We explicitly load languages we expect to see to keep it fast and accurate.
    let languages = vec![
        Language::English,
        Language::French,
        Language::Japanese,
        Language::Chinese,
        Language::Spanish,
        // Add more from Lingua documentation as needed
    ];
    let detector: LanguageDetector = LanguageDetectorBuilder::from_languages(&languages)
        .with_preloaded_language_models()
        .build();

    detector
}

#[deprecated]
pub fn initialize_translation_model() -> Result<TranslationModel, RustBertError> {
    let model_path = PathBuf::from("m2m100_model/rust_model.ot");
    let config_path = PathBuf::from("m2m100_model/config.json");
    let vocab_path = PathBuf::from("m2m100_model/vocab.json");
    let spm_path = PathBuf::from("m2m100_model/sentencepiece.bpe.model");
    let model_resource = LocalResource {
        local_path: model_path,
    };

    let config_resource = LocalResource {
        local_path: config_path,
    };
    let vocab_resource = LocalResource {
        local_path: vocab_path,
    };
    let merges_resource = LocalResource {
        local_path: spm_path,
    };
    let translation_config = TranslationConfig::new(
        rust_bert::pipelines::common::ModelType::M2M100,
        ModelResource::Torch(Box::new(model_resource)),
        config_resource,
        vocab_resource,
        Some(merges_resource), // M2M100 tokenizer often handles this internally
        M2M100SourceLanguages::M2M100_418M,
        [BertLang::English], // Set a default source language
        Device::cuda_if_available(),
    );
    // Create and return the translation model
    Ok(TranslationModel::new(translation_config)?)
}

pub fn initialize_translation_model_from_app_handle(
    app_handle: &tauri::AppHandle,
) -> Result<TranslationModel> {
    let model_path = app_handle
        .path()
        .resolve("m2m100_model/rust_model.ot", BaseDirectory::Resource)?;
    let config_path = app_handle
        .path()
        .resolve("m2m100_model/config.json", BaseDirectory::Resource)?;
    let vocab_path = app_handle
        .path()
        .resolve("m2m100_model/vocab.json", BaseDirectory::Resource)?;
    let spm_path = app_handle.path().resolve(
        "m2m100_model/sentencepiece.bpe.model",
        BaseDirectory::Resource,
    )?;
    let model_resource = LocalResource {
        local_path: model_path,
    };

    let config_resource = LocalResource {
        local_path: config_path,
    };
    let vocab_resource = LocalResource {
        local_path: vocab_path,
    };
    let merges_resource = LocalResource {
        local_path: spm_path,
    };
    let translation_config = TranslationConfig::new(
        rust_bert::pipelines::common::ModelType::M2M100,
        ModelResource::Torch(Box::new(model_resource)),
        config_resource,
        vocab_resource,
        Some(merges_resource), // M2M100 tokenizer often handles this internally
        [
            BertLang::Chinese,
            BertLang::French,
            BertLang::Japanese,
            BertLang::Spanish,
            BertLang::English,
        ],
        [BertLang::English], // Set a default source language
        Device::cuda_if_available(),
    );
    // Create and return the translation model
    Ok(TranslationModel::new(translation_config)?)
}

pub fn process_message(
    text: &str,
    detector: &LanguageDetector,
    model: &TranslationModel,
) -> Result<TranslationResponse, String> {
    // ... (Steps 1, 2, 3 are correct and remain unchanged) ...

    let detected_lang = detector
        .detect_language_of(text)
        .ok_or_else(|| "Unknown Language".to_string())?;

    if detected_lang == Language::English {
        return Ok(TranslationResponse {
            language: detected_lang.to_string(),
            translation: text.to_string(),
        });
    }

    let source_bert_lang = match detected_lang {
        // ... (Mapping logic) ...
        Language::French => Some(BertLang::French),
        Language::Spanish => Some(BertLang::Spanish),
        Language::Japanese => Some(BertLang::Japanese),
        Language::Chinese => Some(BertLang::Chinese),
        _ => None,
    };

    let src = source_bert_lang.ok_or_else(|| {
        "Language supported by detection but not mapped to translator".to_string()
    })?;

    // --- STEP 4: TRANSLATION (Error Handling is fine here) ---
    let output_vec = model
        .translate(&[text], src, BertLang::English)
        // Ensure RustBertError implements Display for easy conversion to String
        .map_err(|err: RustBertError| format!("Bert Translation Error: {:?}", err).to_string())?;

    // 3. Get the first element from the vector.
    // 4. FIX: Use .ok_or_else() to correctly handle the closure as the error producer.
    let translated_text = output_vec
        .first()
        .ok_or_else(|| "Empty Response: Translation vector was empty".to_string())?;

    // --- STEP 5: SUCCESS ---
    Ok(TranslationResponse {
        language: detected_lang.to_string(),
        translation: translated_text.clone(),
    })
}
