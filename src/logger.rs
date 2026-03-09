use tracing::Level;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};


pub fn set_logger() {

    let filter = filter::Targets::new()
    // .with_target("", Level::DEBUG)
    .with_target("agentic_rs", Level::DEBUG)
    .with_target("agentic_core::http", Level::DEBUG)
    .with_target("agentic_core::providers", Level::DEBUG)
    // .with_target("agentic_core_rs::llm", Level::DEBUG)  // Add this
    .with_target("agentic_core::agent", Level::DEBUG)  // Add this
    ;

    tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().compact().pretty())  // Compact format
    .with(filter)
    .init();    

    // tracing_subscriber::fmt()
    // .compact()
    // .pretty()
    // .with_max_level(Level::DEBUG)
    // .init();    
}