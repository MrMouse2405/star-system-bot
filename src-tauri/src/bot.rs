use std::sync::Arc;

use eyre::WrapErr as _;
use tokio::sync::Mutex;
use twitch_api::{
    eventsub::{self, Event, Message, Payload},
    HelixClient,
};
use twitch_oauth2::TwitchToken as _;

use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::{model, websocket, TranslationModelState};

// Define the payload structure we send to the frontend
#[derive(Clone, Serialize, Debug)]
pub struct ChatLogPayload {
    pub user: String,
    pub message: String,
    pub timestamp: String,
}

pub struct Bot {
    pub app_handle: tauri::AppHandle,
    pub client: HelixClient<'static, reqwest::Client>,
    pub token: Arc<Mutex<twitch_oauth2::UserToken>>,
    pub broadcaster: twitch_api::types::UserId,
}

impl Bot {
    pub async fn start(&self) -> Result<(), eyre::Report> {
        // To make a connection to the chat we need to use a websocket connection.
        // This is a wrapper for the websocket connection that handles the reconnects and handles all messages from eventsub.
        let websocket = websocket::ChatWebsocketClient {
            session_id: None,
            token: self.token.clone(),
            client: self.client.clone(),
            connect_url: twitch_api::TWITCH_EVENTSUB_WEBSOCKET_URL.clone(),
            chats: vec![self.broadcaster.clone()],
        };
        let refresh_token = async move {
            let token = self.token.clone();
            let client = self.client.clone();
            // We check constantly if the token is valid.
            // We also need to refresh the token if it's about to be expired.
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                let mut token = token.lock().await;
                if token.expires_in() < std::time::Duration::from_secs(60) {
                    token
                        .refresh_token(&self.client)
                        .await
                        .wrap_err("couldn't refresh token")?;
                }
                token
                    .validate_token(&client)
                    .await
                    .wrap_err("couldn't validate token")?;
            }
            #[allow(unreachable_code)]
            Ok(())
        };
        let ws = websocket.run(|e, ts| async { self.handle_event(e, ts).await });
        futures::future::try_join(ws, refresh_token).await?;
        Ok(())
    }

    async fn handle_event(
        &self,
        event: Event,
        timestamp: twitch_api::types::Timestamp,
    ) -> Result<(), eyre::Report> {
        match event {
            Event::ChannelChatMessageV1(Payload {
                message: Message::Notification(payload),
                subscription,
                ..
            }) => {
                let log = ChatLogPayload {
                    user: payload.chatter_user_name.to_string(),
                    message: payload.message.text.to_string(),
                    timestamp: timestamp.to_string(),
                };
                let _ = self.app_handle.emit("chat-event", &log);
                println!(
                    "[{}] {}: {}",
                    timestamp, payload.chatter_user_name, payload.message.text
                );

                // Clone data for the background thread
                let app_handle = self.app_handle.clone();
                let client = self.client.clone();
                let token_arc = self.token.clone();

                let text = payload.message.text.to_string();
                let chatter_name = payload.chatter_user_name.clone();
                let message_id = payload.message_id.clone();
                let broadcaster_id = subscription.condition.broadcaster_user_id.clone();
                let bot_user_id = subscription.condition.user_id.clone();

                tauri::async_runtime::spawn(async move {
                    let result = model::perform_translation(
                        text.clone(),
                        &app_handle.state::<TranslationModelState>(),
                    )
                    .await;

                    if let Ok(result) = result {
                        if result.language == "English" {
                            tracing::info!("English");
                        } else if result.translation == text {
                            tracing::info!(
                                "Ignored from {}: {}",
                                result.language,
                                result.translation
                            );
                        } else {
                            tracing::info!(
                                "Translated from {}: {}",
                                result.language,
                                result.translation
                            );

                            // Send Reply
                            let token_guard = token_arc.lock().await;

                            let reply_text =
                                format!("(translation) {}: {}", chatter_name, result.translation);

                            if let Err(e) = client
                                .send_chat_message_reply(
                                    &broadcaster_id,
                                    &bot_user_id,
                                    &message_id,
                                    reply_text.as_str(), // ✅ FIX: Use .as_str() here
                                    &*token_guard,
                                )
                                .await
                            {
                                tracing::error!("Failed to send Twitch reply: {}", e);
                            }
                        }
                    }
                });
            }
            Event::ChannelChatNotificationV1(Payload {
                message: Message::Notification(payload),
                ..
            }) => {
                println!(
                    "[{}] {}: {}",
                    timestamp,
                    match &payload.chatter {
                        eventsub::channel::chat::notification::Chatter::Chatter {
                            chatter_user_name: user,
                            ..
                        } => user.as_str(),
                        _ => "anonymous",
                    },
                    payload.message.text
                );
            }
            _ => {}
        }
        Ok(())
    }
}
