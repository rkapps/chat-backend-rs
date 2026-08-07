use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use reqwest::header::HeaderValue;
use rustic_core::{HttpClient, HttpError, HttpResult};
use serde_json::Value;
use tracing::{debug, error, trace};

use crate::{
    client::{
        llm::{CompletionStreamResponse, LlmClient},
        request::CompletionRequest,
        response::CompletionChunkResponse,
    },
    providers::gemini::{
        GEMINI_BASE_URL, chunk::GeminiChunkEvent, helper::to_completion_reponse_token_usage,
        request::GeminiInteractionsRequest,
    },
};

/// [`LlmClient`] implementation for the Google Gemini Interactions API.
///
/// Translates [`CompletionRequest`] into Gemini's wire format, handles SSE streaming,
/// and normalises the response into provider-agnostic types.
#[derive(Debug)]
pub struct GeminiClient {
    pub api_key: String,
    pub base_url: String,
    http_client: HttpClient,
}

impl GeminiClient {
    /// Create a client targeting the default Gemini API endpoint.
    pub fn new(api_key: String) -> Result<Self> {
        Ok(Self {
            api_key,
            base_url: GEMINI_BASE_URL.to_string(),
            http_client: HttpClient::new()?,
        })
    }
}

#[async_trait]
impl LlmClient for GeminiClient {
    async fn complete_with_stream(
        &self,
        request: CompletionRequest,
    ) -> HttpResult<CompletionStreamResponse> {
        let url = format!("{}/v1beta/interactions", self.base_url,);

        let agent_id = request.id.clone();
        let mut headers = reqwest::header::HeaderMap::new();

        let api_key: HeaderValue = self
            .api_key
            .parse()
            .map_err(|_| HttpError::ApiKeyParsingFailed)?;
        headers.insert("x-goog-api-key", api_key);

        let grequest = GeminiInteractionsRequest::new(request)
            .map_err(|e| HttpError::CompletionRequestError(e.to_string()))?;

        grequest.log_info();
        grequest.log_debug();
        grequest.log_trace();

        let body = serde_json::json!(grequest);
        trace!(target: "agent-gemini", body= ?body, "Gemini Completion body");
        let response = self
            .http_client
            .post_stream_request(url, Some(headers), body)
            .await?;

        if response.status() == 400 {
            let error_body = response
                .text()
                .await
                .map_err(|e| HttpError::NetworkError(e.to_string()))?;

            error!("❌ API ERROR BODY: {}", error_body);
            return Err(HttpError::InvalidRequest(error_body));
        }

        let mut event_stream = response.bytes_stream().eventsource();
        let mut pending_tool_calls: HashMap<usize, (String, String, String)> = HashMap::new();

        let stream = async_stream::stream! {

             while let Some(event_result) = event_stream.next().await {
                let event = match event_result {
                    Ok(e) => e,
                    Err(e) => {
                        yield Err(HttpError::NetworkError(e.to_string()));
                        break;
                    }
                };
                trace!(target: "gemini-chunk",
                    event= ?event
                );

                if event.data.contains("[DONE]") {
                    yield Ok(CompletionChunkResponse::default());
                    break;
                }
                let chunk: GeminiChunkEvent = serde_json::from_str(&event.data)
                    .map_err(|e| {
                        HttpError::Other(format!(
                            "GeminiChunkResponse error: {:?} for data {:?}",
                            e, &event.data
                        ))
                    })?;

                // debug!(target: "gemini-chunk",
                //     event_type= ?&event
                // );
                match chunk {
                    GeminiChunkEvent::InteractionCreated { interaction: _, metadata: _ } => {
                    }
                    GeminiChunkEvent::InteractionCompleted { interaction } => {
                        if let Some(interaction) = interaction {
                            debug!(
                                target: "gemini-chunk",
                                interaction= ?&interaction.id,
                                usage= ?&interaction.usage
                            );
                            if let Some(usage) = interaction.usage {
                                let total_usage = to_completion_reponse_token_usage(usage);
                                yield Ok(CompletionChunkResponse::stop(
                                        agent_id.clone(),
                                            interaction.model,
                                            interaction.id,
                                            String::new(),
                                            Some(total_usage),
                                        ))
                            }
                        }
                    }
                    GeminiChunkEvent::StartDelta { index, step} => {
                        if let Some(step) = step {
                            if step.r#type == "function_call" {
                                pending_tool_calls.insert(
                                    index.unwrap_or(0),
                                    (step.id.unwrap_or_default(), step.name.unwrap_or_default(), String::new())
                                );
                            } else {
                                debug!(
                                    target: "gemini-chunk",
                                    step= ?&step
                                );
                            }
                        }
                    }
                    GeminiChunkEvent::StepDelta { index, delta} => {
                        if let Some(delta) = delta {
                            match delta.r#type.as_str() {
                                "arguments_delta" => {
                                    if let Some(entry) = pending_tool_calls.get_mut(&index.unwrap_or(0)) {
                                        entry.2.push_str(&delta.arguments.unwrap_or_default());
                                    }
                                }
                                "thought_summary" => {
                                    // the summary should not be part of thought
                                    // if let Some(content) = delta.content {
                                    //     if let Some(text) = content.text {
                                    //         yield Ok(CompletionChunkResponse::thought(agent_id.clone(), text));
                                    //     }
                                    // }
                                }
                                "thought_signature" => {
                                    if let Some(sig) = delta.signature {
                                        yield Ok(CompletionChunkResponse::thought(agent_id.clone(), sig));
                                    }
                                }
                                "text" => {
                                    // debug!(
                                    //     target: "gemini-chunk",
                                    //     delta= ?&delta
                                    // );
                                    if let Some(text) = delta.text {
                                        yield Ok(CompletionChunkResponse::content(agent_id.clone(), text, String::new()));
                                    }
                                }
                                _ => {
                                    debug!(
                                        target: "gemini-chunk",
                                        delta= ?&delta.r#type

                                    );

                                }
                            }
                        }
                    }
                    GeminiChunkEvent::StopDelta { index, metadata: _} => {

                        if let Some((id, name, args)) = pending_tool_calls.remove(&index.unwrap_or(0)) {
                            let arguments: Value = serde_json::from_str(&args).unwrap_or(Value::Null);
                            yield Ok(CompletionChunkResponse::tool_call(
                                agent_id.clone(),
                                Some(id),
                                Some(name),
                                Some(arguments),
                            ))
                        }
                    }
                    GeminiChunkEvent::Unknown => {
                        debug!(
                            target: "gemini-chunk",
                            "Unknown event type: {:?}", event
                        );
                    }
                }
            }

        };

        Ok(Box::pin(stream))
    }
}
