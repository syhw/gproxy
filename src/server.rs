use axum::{
    extract::State,
    http::StatusCode,
    response::{sse::{Event, Sse}, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use crate::proxy::{proxy_request, get_auth};
use crate::transform::{OpenAIRequest, transform_gemini_to_openai, GeminiResponse};
use std::convert::Infallible;

pub struct ServerState {
    pub port: u16,
    pub host: String,
}

pub async fn start_server(host: &str, port: u16) -> anyhow::Result<()> {
    let state = Arc::new(ServerState {
        port,
        host: host.to_string(),
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/v1/models", get(list_models))
        .route("/v1/chat/completions", post(chat_completions))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    println!("
🚀 Starting Gemini Proxy Server on http://{}", addr);
    println!("═══════════════════════════════════════════════════════
");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> impl IntoResponse {
    let authenticated = match get_auth().await {
        Ok(_) => true,
        Err(_) => false,
    };
    Json(json!({ "status": "ok", "authenticated": authenticated }))
}

async fn list_models() -> impl IntoResponse {
    Json(json!({
        "object": "list",
        "data": [
            { "id": "gemini-2.5-flash", "object": "model", "owned_by": "google" },
            { "id": "gemini-2.5-pro", "object": "model", "owned_by": "google" },
            { "id": "gemini-3-flash-preview", "object": "model", "owned_by": "google" },
            { "id": "gemini-3-pro-preview", "object": "model", "owned_by": "google" },
        ],
    }))
}

async fn chat_completions(
    State(_state): State<Arc<ServerState>>,
    Json(payload): Json<OpenAIRequest>,
) -> impl IntoResponse {
    let model = payload.model.clone();
    let stream = payload.stream.unwrap_or(false);

    match proxy_request(payload).await {
        Ok(res) => {
            if stream {
                let model_clone = model.clone();
                let stream = res.bytes_stream().flat_map(move |chunk| {
                    let model_inner = model_clone.clone();
                    match chunk {
                        Ok(bytes) => {
                            let text = String::from_utf8_lossy(&bytes);
                            let mut events: Vec<Result<Event, Infallible>> = Vec::new();
                            for line in text.lines() {
                                let trimmed = line.trim();
                                if trimmed.starts_with("data: ") {
                                    let data = &trimmed[6..];
                                    if data == "[DONE]" {
                                        // We'll send [DONE] as is or let the client handle it
                                        continue;
                                    }
                                    if let Ok(gemini_res) = serde_json::from_str::<Value>(data) {
                                        let inner = gemini_res.get("response").unwrap_or(&gemini_res);
                                        if let Ok(gemini_response) = serde_json::from_value::<GeminiResponse>(inner.clone()) {
                                            let openai_res = transform_gemini_to_openai(&gemini_response, &model_inner);
                                            for choice in openai_res.choices {
                                                if let Some(content) = choice.message.content {
                                                    let chunk = json!({
                                                        "id": openai_res.id,
                                                        "object": "chat.completion.chunk",
                                                        "created": openai_res.created,
                                                        "model": openai_res.model,
                                                        "choices": [{
                                                            "index": choice.index,
                                                            "delta": { "content": content },
                                                            "finish_reason": choice.finish_reason
                                                        }]
                                                    });
                                                    events.push(Ok(Event::default().data(chunk.to_string())));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            tokio_stream::iter(events)
                        }
                        Err(_) => tokio_stream::iter(vec![Ok(Event::default().data("{\"error\": \"Stream error\"}"))]),
                    }
                });

                Sse::new(stream).into_response()
            } else {
                match res.json::<Value>().await {
                    Ok(gemini_res) => {
                        let inner = gemini_res.get("response").unwrap_or(&gemini_res);
                        match serde_json::from_value::<GeminiResponse>(inner.clone()) {
                            Ok(gemini_response) => {
                                let openai_res = transform_gemini_to_openai(&gemini_response, &model);
                                Json(openai_res).into_response()
                            }
                            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Failed to parse Gemini response: {}", e) }))).into_response()
                        }
                    }
                    Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Failed to get JSON from Gemini: {}", e) }))).into_response()
                }
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}
