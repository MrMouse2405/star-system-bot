use std::num::NonZeroU32;
use std::path::PathBuf;
use std::string::ToString;
use std::sync::{Arc, Mutex};

use crate::RefiningModelState;
use crate::TranslationResponse;

use anyhow::Context;
use anyhow::Result;

use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};

use rust_bert::m2m_100::M2M100SourceLanguages;
use rust_bert::pipelines::common::ModelResource;
use rust_bert::pipelines::translation::{
    Language as BertLang, TranslationConfig, TranslationModel,
};
use rust_bert::resources::LocalResource;
use rust_bert::RustBertError;

use tauri::path::BaseDirectory;
use tauri::Manager;

use tch::Device;

pub fn initialize_lingua() -> LanguageDetector {
    let languages = vec![
        Language::English,
        Language::French,
        Language::Japanese,
        Language::Chinese,
        Language::Spanish,
    ];
    let detector: LanguageDetector = LanguageDetectorBuilder::from_languages(&languages)
        .with_preloaded_language_models()
        .build();

    detector
}

#[deprecated]
#[allow(dead_code)]
pub fn initialize_translation_model() -> Result<TranslationModel, RustBertError> {
    let model_path = PathBuf::from("model/rust_model.ot");
    let config_path = PathBuf::from("model/config.json");
    let vocab_path = PathBuf::from("model/vocab.json");
    let spm_path = PathBuf::from("model/sentencepiece.bpe.model");
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
        Some(merges_resource),
        M2M100SourceLanguages::M2M100_418M,
        [BertLang::English],
        Device::Vulkan,
    );
    Ok(TranslationModel::new(translation_config)?)
}

pub fn initialize_translation_model_from_app_handle(
    app_handle: &tauri::AppHandle,
) -> Result<TranslationModel> {
    let model_path = app_handle
        .path()
        .resolve("model/rust_model.ot", BaseDirectory::Resource)?;
    let config_path = app_handle
        .path()
        .resolve("model/config.json", BaseDirectory::Resource)?;
    let vocab_path = app_handle
        .path()
        .resolve("model/vocab.json", BaseDirectory::Resource)?;
    let spm_path = app_handle
        .path()
        .resolve("model/sentencepiece.bpe.model", BaseDirectory::Resource)?;
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
        Some(merges_resource),
        [
            BertLang::Chinese,
            BertLang::French,
            BertLang::Japanese,
            BertLang::Spanish,
            BertLang::English,
        ],
        [BertLang::English],
        Device::cuda_if_available(),
    );
    Ok(TranslationModel::new(translation_config)?)
}

pub fn initialize_llama_backend() -> Result<LlamaBackend> {
    Ok(LlamaBackend::init()?)
}

pub fn initialize_llm_from_app_handle(
    app_handle: &tauri::AppHandle,
    backend: &LlamaBackend,
) -> Result<LlamaModel> {
    let model_path = app_handle
        .path()
        .resolve("model/Qwen3-8B-Q5_K_M.gguf", BaseDirectory::Resource)
        .context("Failed to resolve path to Qwen model")?;

    let params = LlamaModelParams::default();
    let model = LlamaModel::load_from_file(backend, &model_path, &params)
        .context("Failed to load Qwen model from file")?;

    Ok(model)
}

pub fn process_message(
    text: &str,
    detector: &LanguageDetector,
    translation_model: &Arc<Mutex<TranslationModel>>,
    refining_model: &RefiningModelState,
) -> Result<TranslationResponse, String> {
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
        Language::French => Some(BertLang::French),
        Language::Spanish => Some(BertLang::Spanish),
        Language::Japanese => Some(BertLang::Japanese),
        Language::Chinese => Some(BertLang::Chinese),
        _ => None,
    };

    let src = source_bert_lang.ok_or_else(|| {
        "Language supported by detection but not mapped to translator".to_string()
    })?;

    let translation_model = translation_model
        .lock()
        .map_err(|e| format!("Failed to lock m2m100 model: {}", e))?;

    let output_vec = translation_model
        .translate(&[text], src, BertLang::English)
        // Ensure RustBertError implements Display for easy conversion to String
        .map_err(|err: RustBertError| format!("Bert Translation Error: {:?}", err).to_string())?;

    let translated_text = output_vec
        .first()
        .ok_or_else(|| "Empty Response: Translation vector was empty".to_string())?;

    Ok(TranslationResponse {
        language: detected_lang.to_string(),
        translation: translated_text.to_string(),
    })
}

pub fn refine_with_qwen(
    backend: &LlamaBackend,
    model: &LlamaModel,
    original_lang: &str,
    literal_text: &str,
) -> Result<String> {
    let mut ctx = model
        .new_context(
            backend,
            LlamaContextParams::default().with_n_ctx(NonZeroU32::new(2048)),
        )
        .context("Failed to create llama context")?;

    let prompt = format!(
        r#"<|im_start|>system
Rewrite literal machine-translated chat text from {language} into natural English.
Source is informal online chat (e.g., Twitch comments).
Fix literal slang and idioms.
Keep the same meaning and casual tone.
Do not explain or add information.
Output one natural English sentence only.
If slang is translated literally, replace it with common English internet phrasing.
<|im_end|>

<|im_start|>user
{literal}
<|im_end|>

<|im_start|>assistant"#,
        language = original_lang,
        literal = literal_text
    );

    let tokens_list = model
        .str_to_token(&prompt, AddBos::Always)
        .context("Failed to tokenize prompt")?;

    let mut batch = LlamaBatch::new(2048, 1);

    // Load the prompt into the batch
    let last_index = tokens_list.len() as i32 - 1;
    for (i, token) in tokens_list.iter().enumerate() {
        // We only need logits (predictions) for the very last token of the prompt
        let is_logits = i as i32 == last_index;
        batch.add(*token, i as i32, &[0], is_logits)?;
    }

    // Decode the batch (eval)
    ctx.decode(&mut batch).context("Failed to decode prompt")?;

    // --- Generation Loop ---
    let mut output_string = String::new();
    let max_new_tokens = 4056;
    let mut n_curr = batch.n_tokens(); // Track total tokens processed

    for _ in 0..max_new_tokens {
        let batch_logits_index = if batch.n_tokens() > 1 {
            batch.n_tokens() - 1
        } else {
            0
        };

        let candidates = ctx.candidates_ith(batch_logits_index);

        // Simple Greedy Sampling: Find the token with the highest logit
        let next_token = candidates
            .max_by(|a, b| a.logit().partial_cmp(&b.logit()).unwrap())
            .map(|data| data.id())
            .unwrap_or(model.token_eos());

        // Check for EOS
        if next_token == model.token_eos() {
            break;
        }

        let piece = model.token_to_str(next_token, Special::Tokenize)?;
        output_string.push_str(&piece);

        // Prepare the next batch: it contains only the new token
        batch.clear();
        batch.add(next_token, n_curr, &[0], true)?;

        ctx.decode(&mut batch)?;
        n_curr += 1;
    }

    // Qwen3 produces chain of thought! <think> .. </think>
    // We get rid of it!
    let clean_output = if let Some(index) = output_string.find("</think>") {
        // "index" is where < starts. We add 8 to skip "</think>" length.
        let start_of_text = index + 8;
        if start_of_text < output_string.len() {
            output_string[start_of_text..].to_string()
        } else {
            String::new() // Model stopped right after thinking
        }
    } else {
        // If no </think> tag found, the model likely didn't output a thought block
        // or (edge case) it started thinking but hit max tokens.
        // We return the original string, possibly stripping a leading <think> if needed.
        String::from("<error>")
    };

    Ok(clean_output.trim().to_string())
}
