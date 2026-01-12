use agentic_core_rs::{
    agent::service::AgentService,
    providers::{anthropic, gemini, openai},
};
use agentic_rs::{
    agents::handlers::llm_providers_handler,
    chat::{
        handlers::{
            chat_completion_handler, chat_completion_streaming_handler, create_chat_handler,
            get_all_chats_handler, get_chat_by_id_handler,
        },
        service::ChatService,
        storage::ChatStorage,
    },
    state::AppState,
};
use anyhow::Result;
use axum::{
    Router,
    http::{HeaderValue, Method},
    routing::{get, post},
};
use std::{env, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};
use tower_http::cors::CorsLayer;
use tracing::debug;

#[tokio::main]

async fn main() -> Result<()> {
    agentic_rs::logger::set_logger();

    // initialize storage and the services
    let storage = Mutex::new(ChatStorage::new(
        "agenticdb".to_string(),
        "data/agenticdb".to_string(),
        "chats".to_string(),
    ));

    let mut agent_service = AgentService::new();

    let openai_api_key =
        env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY environment variable not set");

    let _ = agent_service.set_client(openai::completion::LLM, &openai_api_key)?;

    let gemini_api_key =
        env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");
    let _ = agent_service.set_client(gemini::completion::LLM, &gemini_api_key)?;

    let anthropic_api_key =
        env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY environment variable not set");
    let _ = agent_service.set_client(anthropic::completion::LLM, &anthropic_api_key)?;

    let chat_service = ChatService::new(Arc::new(storage));
    let app_state = AppState {
        chat_service: Arc::new(chat_service),
        agent_service: Arc::new(agent_service),
    };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:4200".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/llm/providers", get(llm_providers_handler))
        .route("/chats", get(get_all_chats_handler))
        .route("/chats/{id}", get(get_chat_by_id_handler))
        .route("/chats/create", post(create_chat_handler))
        .route("/chats/completion", post(chat_completion_handler))
        .route("/chats/completion_streaming", post(chat_completion_streaming_handler))
        .layer(cors)
        .with_state(app_state) // Shared state
        ;

    let listener = TcpListener::bind("127.0.0.1:3001").await.unwrap();
    println!("🚀 Listening on http://127.0.0.1:3001");

    axum::serve(listener, app)
        .with_graceful_shutdown(handle_shutdown())
        .await
        .unwrap();

    debug!("Shutdown complete.");

    Ok(())
}

async fn handle_shutdown() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    debug!("handled shutdown");
}
