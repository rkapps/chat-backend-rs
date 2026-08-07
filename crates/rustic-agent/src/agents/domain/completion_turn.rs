/// A single completed exchange: the user prompt sent to an agent and the assistant reply received.
///
/// [`PipeLineAgent`](super::runner::PipeLineAgent) accumulates these across stages to build the
/// growing conversation history that is replayed to the orchestrator on each decision turn.
#[derive(Debug, Clone)]
pub struct CompletionTurn {
    /// Position of this turn in the pipeline (1-based).
    pub sequence: u32,
    pub user_content: String,
    pub response_content: String,
    /// Provider-assigned ID for the assistant response; used for multi-turn context continuations.
    pub response_id: Option<String>,
}

impl CompletionTurn {}
