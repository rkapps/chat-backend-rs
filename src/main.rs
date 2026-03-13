use agentic_core::agent::service::AgentService;
use anyhow::Result;
use axum::{
    Router,
    http::{HeaderValue, Method},
    routing::{get, post},
};
use chat_backend_rs::{
    chat::{
        handlers::{
            chat_completion_handler, chat_completion_streaming_handler, create_chat_handler,
            get_all_chats_handler, get_chat_by_id_handler, llm_providers_handler,
            save_streaming_message_handler,
        },
        service::ChatService,
        storage::ChatStorage,
    },
    logger,
    state::AppState,
};
use std::{env, sync::Arc};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::debug;

#[tokio::main]

async fn main() -> Result<()> {
    logger::set_logger();

    // initialize storage and the services
    let storage = ChatStorage::new(
        "agenticdb".to_string(),
        "data/agenticdb".to_string(),
        "chats".to_string(),
    )
    .await?;

    let openai_api_key =
        env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY environment variable not set");
    let gemini_api_key =
        env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");
    let anthropic_api_key =
        env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY environment variable not set");

    let agent_service = AgentService::new();
    let openai_agent = agent_service
        .builder()
        .with_openai(&openai_api_key)?
        .build()?;
    let gemini_agent = agent_service
        .builder()
        .with_gemini(&gemini_api_key)?
        .build()?;
    let anthropic_agent = agent_service
        .builder()
        .with_anthropic(&anthropic_api_key)?
        .build()?;

    let mut chat_service = ChatService::new(storage);
    chat_service.add_agent(openai_agent);
    chat_service.add_agent(gemini_agent);
    chat_service.add_agent(anthropic_agent);
    chat_service.add_llm_provider(agent_service.get_llm_providers());

    let app_state = AppState {
        chat_service: Arc::new(chat_service),
    };

    let origins = [
        "http://localhost:4200".parse::<HeaderValue>().unwrap(),
        "http://localhost:4201".parse::<HeaderValue>().unwrap(),
        "http://localhost:4202".parse::<HeaderValue>().unwrap(),
    ];

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/llm/providers", get(llm_providers_handler))
        .route("/chats", get(get_all_chats_handler))
        .route("/chats/{id}", get(get_chat_by_id_handler))
        .route("/chats/create", post(create_chat_handler))
        .route("/chats/completion", post(chat_completion_handler))
        .route("/chats/completion_streaming", post(chat_completion_streaming_handler))
        .route("/chats/save_streaming_message", post(save_streaming_message_handler))
        .layer(cors)
        .with_state(app_state) // Shared state
        ;

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("🚀 Listening on http://127.0.0.1:8080");

    axum::serve(listener, app)
        .with_graceful_shutdown(handle_shutdown())
        .await
        .unwrap();

    Ok(())
}

async fn handle_shutdown() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    debug!("handled shutdown");
}
