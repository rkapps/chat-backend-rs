use agentic_rs::{handlers::{
    chat::{chat_completion_handler, chat_create_handler, get_chat_by_id_handler, get_chats_handler},
    llm::llm_providers_handler,
}, storage::{self, load_chatmap_from_disk}};
use axum::{
    Router, http::{HeaderValue, Method}, routing::{get, post}
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::debug;

#[tokio::main]

async fn main() {
    agentic_rs::logger::set_logger();

    //load chats 
    load_chatmap_from_disk().await.expect("Error loading chats from disp");

    let cors = CorsLayer::new()
    .allow_origin("http://localhost:4200".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

        
    let app = Router::new()
        .route("/llm/providers", get(llm_providers_handler))
        .route("/chats", get(get_chats_handler))
        .route("/chats/{id}", get(get_chat_by_id_handler))
        .route("/chats/create", post(chat_create_handler))
        .route("/chats/completion", post(chat_completion_handler))
        .layer(cors);

    let listener = TcpListener::bind("127.0.0.1:3001").await.unwrap();
    println!("🚀 Listening on http://127.0.0.1:3001");

    axum::serve(listener, app)
        .with_graceful_shutdown(handle_shutdown())
        .await
        .unwrap();

    debug!("Saving maps to disk...");
    storage::save_chatmap_to_disk().await;
    debug!("Shutdown complete.");

}

async fn handle_shutdown() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    debug!("handled shutdown");
}
