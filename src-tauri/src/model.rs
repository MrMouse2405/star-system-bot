use std::env;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use std::num::NonZeroU32;

use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};

use tauri::path::BaseDirectory;
use tauri::Manager;

use crate::slang_fr;
use crate::slang_jp;
use crate::slang_zh;
use crate::TranslationModelState;
use crate::TranslationResponse;

const QWEN_MODEL_NAME: &str = "Qwen3-1.7B-Q8_0.gguf";

// --- WRAPPER FOR THREAD SAFETY ---
// We wrap LlamaContext to implement Send + Sync manually.
// This is safe because we guard access with a Mutex in main.rs.
pub struct ThreadSafeContext(pub LlamaContext<'static>);

unsafe impl Send for ThreadSafeContext {}
unsafe impl Sync for ThreadSafeContext {}
// ---------------------------------

pub fn initialize_lingua() -> LanguageDetector {
    let languages = vec![
        Language::English,
        Language::French,
        Language::Japanese,
        Language::Chinese,
    ];
    LanguageDetectorBuilder::from_languages(&languages)
        .with_preloaded_language_models()
        .build()
}

pub fn initialize_llama_backend() -> Result<LlamaBackend> {
    Ok(LlamaBackend::init()?)
}

// We use unsafe to extend the lifetime to 'static because we know
// the Model is stored in an Arc alongside the Context, so it won't drop early.
pub fn initialize_llama_context(
    backend: &LlamaBackend,
    model: &LlamaModel,
) -> Result<ThreadSafeContext> {
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(Some(NonZeroU32::new(2048).unwrap()))
        .with_n_batch(2048)
        .with_n_ubatch(2048)
        .with_n_threads(4)
        .with_n_threads_batch(4);

    let ctx = model
        .new_context(backend, ctx_params)
        .context("Failed to create llama context")?;

    // SAFETY: We are forcefully extending the lifetime to 'static.
    // This is necessary to store it in the Tauri state.
    // It remains safe as long as 'model' (in Arc) lives as long as 'ctx'.
    let static_ctx: LlamaContext<'static> = unsafe { std::mem::transmute(ctx) };

    Ok(ThreadSafeContext(static_ctx))
}

// ---------------------------------------------------------------------------
// OPTION A: THE "FLATPAK HACK" (Active only when --features flatpak is used)
// ---------------------------------------------------------------------------
#[cfg(feature = "flatpak")]
pub fn initialize_llm_from_app_handle(
    app_handle: &tauri::AppHandle,
    backend: &LlamaBackend,
) -> Result<LlamaModel> {
    println!("DEBUG: Initializing LLM using FLATPAK logic");

    // 1. Get the path of the actual running binary inside Flatpak (/app/bin/start-bot)
    let exe_path = env::current_exe().context("Failed to get current exe path")?;

    // 2. Get the parent folder (/app/bin)
    let exe_dir = exe_path.parent().context("Failed to get exe parent dir")?;

    // 3. Manually construct the path to the model (/app/bin/model/Qwen...)
    let model_path = exe_dir.join("model").join(QWEN_MODEL_NAME);

    println!("DEBUG: Looking for model at: {:?}", model_path);

    if !model_path.exists() {
        return Err(anyhow::anyhow!("Model file not found at: {:?}", model_path));
    }

    let params = LlamaModelParams::default().with_n_gpu_layers(999);
    let model = LlamaModel::load_from_file(backend, &model_path, &params)
        .context("Failed to load Qwen model from file")?;

    Ok(model)
}

// ---------------------------------------------------------------------------
// OPTION B: THE "STANDARD TAURI" WAY (Active by default)
// ---------------------------------------------------------------------------
#[cfg(not(feature = "flatpak"))]
pub fn initialize_llm_from_app_handle(
    app_handle: &tauri::AppHandle,
    backend: &LlamaBackend,
) -> Result<LlamaModel> {
    println!("DEBUG: Initializing LLM using STANDARD TAURI logic");

    let model_path = app_handle
        .path()
        .resolve(
            format!("model/{}", QWEN_MODEL_NAME),
            BaseDirectory::Resource,
        )
        .context("Failed to resolve path to Qwen model")?;

    let params = LlamaModelParams::default().with_n_gpu_layers(999);
    let model = LlamaModel::load_from_file(backend, &model_path, &params)
        .context("Failed to load Qwen model from file")?;

    Ok(model)
}

pub fn localize_with_qwen(
    model: &LlamaModel,
    wrapped_ctx: &mut ThreadSafeContext, // Accept the wrapper
    source_lang: &str,
    raw_text: &str,
) -> Result<String> {
    let ctx = &mut wrapped_ctx.0; // Access internal context

    ctx.clear_kv_cache();

    let n_ctx = NonZeroU32::new(2048).unwrap();

    let prompt = format!(
        //         r#"<|im_start|>system
        // Localize {language} gaming chat to natural, informal English.
        // Adapt slang/idioms to Western gaming terms (e.g., 'lol', 'choke', 'clutch').
        // Maintain the user's tone. If the text only includes link, ignore it and
        // reply with '<ignore>'. If the text is unclear to translate, reply with
        // '<ignore>'. If the translation is too harsh, tone it down.
        // Otherwise, output translation only.<|im_end|>
        // <|im_start|>user
        // {raw_input}
        // <|im_end|>
        // <|im_start|>assistant"#,
        r#"<|im_start|>system
If the text is in English, reply with '<@>' exactly.
Localize gaming chat to natural, informal English.
Adapt slang/idioms to Western gaming terms (e.g., 'lol', 'choke', 'clutch').
Maintain the user's tone. If the text only includes link, ignore it and
reply with '<@>' exactly. If the text is unclear to translate, reply with
'<@>' exactly. If the translation is too harsh, tone it down. 
Otherwise, output translation or '<@>' exactly only.<|im_end|>
<|im_start|>user
{raw_input}
<|im_end|>
<|im_start|>assistant"#,
        // language = source_lang,
        raw_input = raw_text
    );

    let prompt_tokens = model
        .str_to_token(&prompt, AddBos::Always)
        .context("Failed to tokenize prompt")?;

    let mut batch = LlamaBatch::new(2048, 1);

    let last_index = prompt_tokens.len() as i32 - 1;
    for (i, token) in prompt_tokens.iter().enumerate() {
        let is_last = i as i32 == last_index;
        batch.add(*token, i as i32, &[0], is_last)?;
    }

    ctx.decode(&mut batch).context("Failed to decode prompt")?;

    let mut response_bytes = Vec::<u8>::with_capacity(4096);
    let max_new_tokens = 2048;
    let mut n_curr = batch.n_tokens();

    for _ in 0..max_new_tokens {
        if n_curr as u32 >= n_ctx.get() {
            break;
        }

        let last_token_idx = batch.n_tokens() - 1;
        let candidates = ctx.candidates_ith(last_token_idx);

        let next_token = candidates
            .max_by(|a, b| a.logit().partial_cmp(&b.logit()).unwrap())
            .map(|data| data.id())
            .unwrap_or(model.token_eos());

        if next_token == model.token_eos() {
            break;
        }

        let piece = model.token_to_bytes(next_token, Special::Tokenize)?;
        response_bytes.extend(piece);

        batch.clear();
        batch.add(next_token, n_curr, &[0], true)?;

        ctx.decode(&mut batch)?;
        n_curr += 1;
    }

    let full_response = String::from_utf8_lossy(&response_bytes).to_string();

    let clean_output = if let Some(_) = full_response.find("<@>") {
        String::new()
    } else if let Some(end_tag_pos) = full_response.find("</think>") {
        let start_of_text = end_tag_pos + 8;
        if start_of_text < full_response.len() {
            full_response[start_of_text..].to_string()
        } else {
            String::new()
        }
    } else {
        if let Some(_) = full_response.find("<think>") {
            return Ok(String::from("<error: I thought too hard>"));
        }
        String::new()
    };

    Ok(clean_output.trim().to_string())
}

pub async fn perform_translation(
    text: String,
    state: &TranslationModelState,
) -> Result<TranslationResponse, String> {
    // FAST PATH: Check for slang/abbreviations immediately
    if is_universal_slang(&text) {
        return Ok(TranslationResponse {
            language: "English".into(),
            translation: text,
        });
    }

    // Check if it's English!
    let detected_lang = state
        .detector
        .detect_language_of(&text)
        .ok_or_else(|| "Unknown Language".to_string())?;

    //  If it is, then we skip!
    let processed_text = match detected_lang {
        Language::Chinese => slang_zh::normalize_mandarin_slang(&text),
        Language::Japanese => slang_jp::normalize_japanese_slang(&text),
        Language::French => slang_fr::normalize_french_slang(&text),
        Language::English => {
            return Ok(TranslationResponse {
                language: "English".into(),
                translation: text,
            })
        }
        _ => text.clone(),
    };

    let language_label = detected_lang.to_string();

    // We clone the Arcs here so they can be moved into the spawn_blocking closure
    let llm_state = state.llm_state.clone();
    let semaphore = state.semaphore.clone();

    // Acquire semaphore (Async wait)
    let _permit = semaphore
        .acquire_owned()
        .await
        .map_err(|e| format!("Semaphore Error: {}", e))?;

    // Run inference (Blocking thread)
    let translation = tauri::async_runtime::spawn_blocking(move || {
        let mut ctx = {
            let mut pool = llm_state
                .context_pool
                .lock()
                .map_err(|_| "Poisoned lock")
                .unwrap();
            pool.pop().expect("Semaphore logic failed: Pool was empty!")
        };

        let result =
            localize_with_qwen(&llm_state.model, &mut ctx, &language_label, &processed_text);

        {
            let mut pool = llm_state
                .context_pool
                .lock()
                .map_err(|_| "Poisoned lock")
                .unwrap();
            pool.push(ctx);
        }

        result
    })
    .await
    .map_err(|e| format!("Task Join Error: {}", e))?
    .map_err(|e| format!("LLM Inference Error: {}", e))?;

    Ok(TranslationResponse {
        language: detected_lang.to_string(),
        translation,
    })
}

fn is_universal_slang(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }

    // We split by whitespace to handle messages like "LUL LUL LUL"
    text.split_whitespace().all(|token| {
        // Remove common punctuation to handle "LMAO!" or "WTF?"
        // This will also remove emojis!
        // and emoticons :)
        let clean_token: String = token.chars().filter(|c| c.is_alphanumeric()).collect();

        if clean_token.is_empty() {
            return true;
        }

        // Check against a hardcoded list of universal slang
        match clean_token.to_uppercase().as_str() {
            "LMAO" | "LMFAO" | "LOL" | "ROFL" | "LUL" | "KEKW" | "OMEGALUL" | "POG" | "POGGERS"
            | "POGCHAMP" | "KAPPA" | "MONKAW" | "MONKAS" | "PEPELAUGH" | "SADGE" | "BRUH"
            | "WTF" | "OMG" | "IDK" | "XD" | "XDD" | "HA" | "HAHA" | "HAHAHA" | "JAJA"
            | "JAJAJA" | "MDR" | "L" | "FTFY" | "ERM" => true,
            _ => false,
        }
    })
}
