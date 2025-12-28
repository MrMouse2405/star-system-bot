use lingua::LanguageDetector;
use llama_cpp_2::{llama_backend::LlamaBackend, model::LlamaModel};
use reqwest::header::InvalidHeaderValue;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tokio::sync::Semaphore;
use twitch_api::client::ClientDefault;
use twitch_api::{client::ReqwestClientDefaultError, HelixClient};
use twitch_oauth2::{AccessToken, DeviceUserTokenBuilder, Scope, TwitchToken as _, UserToken};

mod bot;
mod model;
mod slang_fr;
mod slang_jp;
mod slang_zh;
mod websocket;

const STORE_PATH: &str = "configs.json";
const CLIENT_ID_KEY: &str = "client_id";
const CLIENT_SECRET_KEY: &str = "client_secret";
const CONTEXT_THREADS: usize = 20;

#[allow(unused)]
struct RefiningModelState {
    backend: Arc<LlamaBackend>,
    model: Arc<LlamaModel>,
    context_pool: Mutex<Vec<model::ThreadSafeContext>>,
}

struct TranslationModelState {
    detector: LanguageDetector,
    llm_state: Arc<RefiningModelState>,
    semaphore: Arc<Semaphore>,
}

struct TwitchBotState {
    client_id: Mutex<Option<String>>,
    client_secret: Mutex<Option<String>>,
}

struct AuthorizationFlow {
    client_id: Mutex<Option<String>>,
    builder: Mutex<Option<DeviceUserTokenBuilder>>,
}

struct JoinedChannelState {
    join_handle: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TranslationResponse {
    language: String,
    translation: String,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            translate,
            get_token,
            wait_for_token,
            check_auth_status,
            join_channel,
            leave_channel,
            is_in_channel
        ])
        .setup(move |app| {
            color_eyre::install()?;
            tracing_subscriber::fmt::fmt()
                .with_writer(std::io::stderr)
                .init();

            let app_handle = app.handle();

            let llama_backend = Arc::new(
                model::initialize_llama_backend().expect("Failed to load llamacpp backend!"),
            );

            let llm = Arc::new(
                model::initialize_llm_from_app_handle(&app_handle, &llama_backend)
                    .expect("failed to load qwen3 model!"),
            );

            let mut contexts = Vec::new();
            for _ in 0..5 {
                let ctx = model::initialize_llama_context(&llama_backend, &llm)
                    .expect("Failed to create context");
                contexts.push(ctx);
            }

            app.manage(TranslationModelState {
                detector: model::initialize_lingua(),
                llm_state: Arc::new(RefiningModelState {
                    backend: llama_backend,
                    model: llm,
                    context_pool: Mutex::new(contexts),
                }),
                semaphore: Arc::new(Semaphore::new(CONTEXT_THREADS)),
            });

            let store = app.store(STORE_PATH)?;

            // Initialize Twitch State
            let twitch_bot_state = TwitchBotState {
                client_id: Mutex::new(None),
                client_secret: Mutex::new(None),
            };

            // Load from Store if exists
            let client_id = store.get(CLIENT_ID_KEY);
            if let Some(value) = client_id {
                if let serde_json::Value::String(value) = value {
                    *twitch_bot_state.client_id.lock().unwrap() = Some(value.clone());
                }
            }

            let client_secret = store.get(CLIENT_SECRET_KEY);
            if let Some(value) = client_secret {
                if let serde_json::Value::String(value) = value {
                    *twitch_bot_state.client_secret.lock().unwrap() = Some(value.clone());
                }
            }

            app.manage(twitch_bot_state);
            app.manage(AuthorizationFlow {
                client_id: Mutex::new(None),
                builder: Mutex::new(None),
            });
            app.manage(JoinedChannelState {
                join_handle: Mutex::new(None),
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn translate(
    text: String,
    state: tauri::State<'_, TranslationModelState>,
) -> Result<TranslationResponse, String> {
    model::perform_translation(text, &state).await
}

#[tauri::command]
async fn check_auth_status(state: tauri::State<'_, TwitchBotState>) -> Result<bool, String> {
    // 1. Lock mutexes to get values safely
    let client_id = state.client_id.lock().map_err(|_| "Poisoned lock")?.clone();
    let client_secret = state
        .client_secret
        .lock()
        .map_err(|_| "Poisoned lock")?
        .clone();

    if let (Some(_), Some(access_token)) = (client_id, client_secret) {
        // 2. Create a client to test the token
        let client: HelixClient<reqwest::Client> = twitch_api::HelixClient::with_client(
            ClientDefault::default_client_with_name(Some(
                "star-system-bot"
                    .parse()
                    .map_err(|e: InvalidHeaderValue| e.to_string())?,
            ))
            .map_err(|e: ReqwestClientDefaultError| e.to_string())?,
        );

        let token =
            UserToken::from_existing(&client, AccessToken::new(access_token), None, None).await;

        match token {
            Ok(t) => {
                if t.validate_token(&client).await.is_ok() {
                    return Ok(true);
                }
            }
            Err(_) => return Ok(false),
        }
    }

    Ok(false)
}

#[tauri::command]
async fn get_token(
    client_id: String,
    state: tauri::State<'_, AuthorizationFlow>,
) -> Result<String, String> {
    let client: HelixClient<reqwest::Client> = twitch_api::HelixClient::with_client(
        ClientDefault::default_client_with_name(Some(
            "star-system-bot"
                .parse()
                .map_err(|e: InvalidHeaderValue| e.to_string())?,
        ))
        .map_err(|e: ReqwestClientDefaultError| e.to_string())?,
    );

    let mut builder = twitch_oauth2::tokens::DeviceUserTokenBuilder::new(
        client_id.clone(),
        vec![Scope::UserReadChat, Scope::UserWriteChat],
    );

    let code = builder.start(&client).await.map_err(|e| e.to_string())?;
    let auth_url = code.verification_uri.to_string();

    *state.builder.lock().map_err(|_| "Failed to lock mutex")? = Some(builder);
    *state.client_id.lock().map_err(|_| "Failed to lock mutex")? = Some(client_id);

    Ok(auth_url)
}

#[tauri::command]
async fn wait_for_token(
    app: tauri::AppHandle,
    auth_flow: tauri::State<'_, AuthorizationFlow>,
    bot_state: tauri::State<'_, TwitchBotState>,
) -> Result<(), String> {
    // 1. Retrieve Client ID from auth flow state
    let client_id_str = {
        let mut guard = auth_flow
            .client_id
            .lock()
            .map_err(|_| "Failed to lock mutex")?;
        guard.take().ok_or("Authentication flow has not started")?
    };

    // 2. Retrieve Builder
    let mut builder = {
        let mut guard = auth_flow
            .builder
            .lock()
            .map_err(|_| "Failed to lock mutex")?;
        guard.take().ok_or("Authentication flow has not started")?
    };

    let client = reqwest::Client::new();

    // 3. Wait for User to click Accept in Browser
    let token = builder
        .wait_for_code(&client, tokio::time::sleep)
        .await
        .map_err(|e| e.to_string())?;

    let access_token = token.access_token.secret().to_string();

    // 4. Update the TwitchBotState (The Fix: Lock, then Assign)
    {
        let mut id_lock = bot_state.client_id.lock().map_err(|_| "Failed lock")?;
        *id_lock = Some(client_id_str.clone());

        let mut secret_lock = bot_state.client_secret.lock().map_err(|_| "Failed lock")?;
        *secret_lock = Some(access_token.clone());
    }

    // 5. Persist to Disk
    let store = app.store(STORE_PATH).map_err(|err| err.to_string())?;
    store.set(CLIENT_ID_KEY, client_id_str);
    store.set(CLIENT_SECRET_KEY, access_token);
    let _ = store.save(); // Don't forget to save!

    Ok(())
}

#[tauri::command]
async fn is_in_channel(bot_state: tauri::State<'_, JoinedChannelState>) -> Result<bool, String> {
    if let Some(_) = *bot_state
        .join_handle
        .lock()
        .map_err(|err| err.to_string())?
    {
        return Ok(true);
    }

    Ok(false)
}

#[tauri::command]
async fn join_channel(
    app: tauri::AppHandle,
    broadcaster_login: String,
    state: tauri::State<'_, TwitchBotState>,
    bot_state: tauri::State<'_, JoinedChannelState>,
) -> Result<(), String> {
    tracing::info!("Joining channel {}", &broadcaster_login);

    // 1. Extract Credentials properly using Locks
    let (_, access_token) = {
        let id_lock = state.client_id.lock().map_err(|_| "Lock poisoned")?;
        let secret_lock = state.client_secret.lock().map_err(|_| "Lock poisoned")?;

        match (&*id_lock, &*secret_lock) {
            (Some(id), Some(secret)) => (id.clone(), secret.clone()),
            _ => return Err("Credentials not found. Please log in again.".to_string()),
        }
    };

    let client: HelixClient<reqwest::Client> = twitch_api::HelixClient::with_client(
        ClientDefault::default_client_with_name(Some(
            "star-system-bot"
                .parse()
                .map_err(|e: InvalidHeaderValue| e.to_string())?,
        ))
        .map_err(|e: ReqwestClientDefaultError| e.to_string())?,
    );

    let token: UserToken =
        UserToken::from_existing(&client, AccessToken::new(access_token), None, None)
            .await
            .map_err(|e| e.to_string())?;

    // We need to know the numeric ID of the channel we want to join
    let broadcaster_username: twitch_api::types::UserName =
        broadcaster_login
            .as_str()
            .try_into()
            .map_err(|_| "Invalid broadcaster username")?;

    let user = client
        .get_user_from_login(&broadcaster_username, &token)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Broadcaster not found")?;

    let broadcaster_id = user.id;

    let bot = bot::Bot {
        app_handle: app.clone(),
        client,
        token: Arc::new(tokio::sync::Mutex::new(token)),
        broadcaster: broadcaster_id,
    };

    // We must spawn this because bot.start() is an infinite loop
    *bot_state
        .join_handle
        .lock()
        .map_err(|_| "Failed to lock mutex")? = Some(tauri::async_runtime::spawn(async move {
        println!("Bot starting background task...");
        if let Err(e) = bot.start().await {
            eprintln!("Bot crashed: {}", e);
        }
    }));

    tracing::info!("Joined channel {}", &broadcaster_login);

    Ok(())
}

#[tauri::command]
async fn leave_channel(bot_state: tauri::State<'_, JoinedChannelState>) -> Result<(), String> {
    tracing::info!("Leaving channel");

    let maybe_handle = {
        let mut guard = bot_state
            .join_handle
            .lock()
            .map_err(|_| "Failed to lock mutex")?;

        guard.take()
    };

    if let Some(handle) = maybe_handle {
        handle.abort();
        tracing::info!("Left channel");
        Ok(())
    } else {
        Err("Bot is currently not in any channel!".to_string())
    }
}
