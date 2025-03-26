use crate::tokenizer::exceeds_token_limit;
use anyhow::{anyhow, Result};
use futures::StreamExt;
use rig::agent::Agent;
use rig::completion::{AssistantContent, Prompt};
use rig::completion::Message;
use rig::message::{ToolResultContent, UserContent};
use rig::providers::anthropic::completion::CompletionModel;
use rig::streaming::StreamingChoice;
use rig::streaming::StreamingCompletion;
use rig::OneOrMany;
use serde::Deserialize;
use serde::Serialize;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use uuid;
use serde_json;
use crate::http::state::AppState;

#[derive(Serialize, Debug, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum StreamResponse {
    Message(String),
    ToolCall {
        id: String,
        name: String,
        params: String,
    },
    ToolResult {
        id: String,
        name: String,
        result: String,
    },
    Error(String),
}

pub struct ReasoningLoop {
    agent: Arc<Agent<CompletionModel>>,
    stdout: bool,
}

impl ReasoningLoop {
    pub fn new(agent: Arc<Agent<CompletionModel>>) -> Self {
        Self {
            agent,
            stdout: true,
        }
    }

    pub async fn stream(
        &self,
        prompt: String,
        messages: Vec<Message>,
        tx: Option<Sender<StreamResponse>>,
    ) -> Result<Vec<Message>> {
        if tx.is_none() && !self.stdout {
            panic!("enable stdout or provide tx channel");
        }

        // Simple character-based check for token limit
        if exceeds_token_limit(&prompt, &messages, 40_000) {
            return Err(anyhow::anyhow!(
                "Ahoy! Context is getting long, please start a new conversation",
            ));
        }

        let mut current_messages = messages.clone();
        let agent = self.agent.clone();
        let stdout = self.stdout;

        // Start with the user's original prompt
        let mut next_input = Message::user(prompt.clone());
        let mut is_first_iteration = true;

        'outer: loop {
            let mut current_response = String::new();

            // Stream using the next input (original prompt or tool result)
            let mut stream = match agent
                .stream_completion(
                    next_input.clone(),
                    current_messages.clone(),
                )
                .await?
                .stream()
                .await
            {
                Ok(stream) => stream,
                Err(e) => {
                    tracing::error!("Error: failed to stream chat: {}", e);
                    return Err(anyhow::anyhow!(
                        "failed to stream chat: {}",
                        e
                    ));
                }
            };

            // Only add the original user prompt to history on the first iteration
            if is_first_iteration {
                current_messages.push(Message::User {
                    content: OneOrMany::one(UserContent::text(
                        prompt.clone(),
                    )),
                });
                is_first_iteration = false;
            } else {
                // For subsequent iterations, add the tool result message to history
                current_messages.push(next_input.clone());
            }

            while let Some(chunk) = stream.next().await {
                match chunk? {
                    StreamingChoice::Message(text) => {
                        if stdout {
                            print!("{}", text);
                            std::io::stdout().flush()?;
                        } else if let Some(tx) = &tx {
                            tx.send(StreamResponse::Message(text.clone()))
                                .await
                                .map_err(|e| {
                                    anyhow::anyhow!(
                                        "failed to send message: {}",
                                        e
                                    )
                                })?;
                        }
                        current_response.push_str(&text);
                    }
                    StreamingChoice::ToolCall(name, tool_id, params) => {
                        // Add the assistant's response up to this point with the tool call
                        if !current_response.is_empty() {
                            current_messages.push(Message::Assistant {
                                content: OneOrMany::one(
                                    AssistantContent::text(
                                        current_response.clone(),
                                    ),
                                ),
                            });
                            current_response.clear();
                        }

                        // Add the tool use message from the assistant
                        current_messages.push(Message::Assistant {
                            content: OneOrMany::one(
                                AssistantContent::tool_call(
                                    tool_id.clone(),
                                    name.clone(),
                                    params.clone(),
                                ),
                            ),
                        });

                        if let Some(tx) = &tx {
                            tx.send(StreamResponse::ToolCall {
                                id: tool_id.clone(),
                                name: name.clone(),
                                params: params.to_string(),
                            })
                            .await
                            .map_err(|e| {
                                anyhow::anyhow!(
                                    "failed to send tool call: {}",
                                    e
                                )
                            })?;
                        }

                        // Call the tool and get result
                        let result = self
                            .agent
                            .tools
                            .call(&name, params.to_string())
                            .await;

                        if stdout {
                            println!("Tool result: {:?}", result);
                        }

                        // Create the tool result message to use directly as next input
                        let result_str = match &result {
                            Ok(content) => content.to_string(),
                            Err(err) => err.to_string(),
                        };

                        // Create the tool result message to be used as the next input
                        next_input = Message::User {
                            content: OneOrMany::one(
                                UserContent::tool_result(
                                    tool_id.clone(),
                                    OneOrMany::one(ToolResultContent::text(
                                        result_str.clone(),
                                    )),
                                ),
                            ),
                        };

                        if let Some(tx) = &tx {
                            tx.send(StreamResponse::ToolResult {
                                id: tool_id,
                                name,
                                result: result_str,
                            })
                            .await
                            .map_err(|e| {
                                anyhow::anyhow!(
                                    "failed to send tool call: {}",
                                    e
                                )
                            })?;
                        }

                        continue 'outer;
                    }
                }
            }

            // Add any remaining response to messages
            if !current_response.is_empty() {
                current_messages.push(Message::Assistant {
                    content: OneOrMany::one(AssistantContent::text(
                        current_response,
                    )),
                });
            }

            // If we get here, there were no tool calls in this iteration
            break;
        }

        Ok(current_messages)
    }

    pub fn with_stdout(mut self, enabled: bool) -> Self {
        self.stdout = enabled;
        self
    }
}

/// Handles analysis of a topic via LunarCrush
/// Returns a tool call result with LunarCrush analysis
pub async fn handle_lunarcrush_analysis(
    lunarcrush_api: Arc<crate::lunarcrush::LunarCrushApi>,
    topic: &str,
) -> Result<StreamResponse, anyhow::Error> {
    match lunarcrush_api.research_topic(topic).await {
        Ok(response) => {
            Ok(StreamResponse::ToolResult {
                id: uuid::Uuid::new_v4().to_string(),
                name: "LUNARCRUSH_ANALYSIS".to_string(),
                result: serde_json::to_string(&response).unwrap_or_default(),
            })
        }
        Err(e) => {
            tracing::error!("Error in LunarCrush analysis: {}", e);
            Ok(StreamResponse::Error(format!(
                "Failed to analyze topic with LunarCrush: {}",
                e
            )))
        }
    }
}

/// Handles analysis of a token via Faster100x
/// Returns a tool call result with Faster100x analysis
pub async fn handle_faster100x_analysis(
    faster100x_api: Arc<crate::faster100x::Faster100xApi>,
    token_address: &str,
) -> Result<StreamResponse, anyhow::Error> {
    match faster100x_api.get_faster100x_data(token_address).await {
        Ok(response) => {
            Ok(StreamResponse::ToolResult {
                id: uuid::Uuid::new_v4().to_string(),
                name: "FASTER100X_ANALYSIS".to_string(),
                result: serde_json::to_string(&response).unwrap_or_default(),
            })
        }
        Err(e) => {
            tracing::error!("Error in Faster100x analysis: {}", e);
            Ok(StreamResponse::Error(format!(
                "Failed to analyze token with Faster100x: {}",
                e
            )))
        }
    }
}

pub async fn handle_message(
    message: &str,
    state: &AppState,
    _locale: &str,
) -> Result<Vec<Message>, anyhow::Error> {
    let mut current_messages = Vec::new();

    let response = state
        .agent
        .prompt(message)
        .await
        .map_err(|e| anyhow!("Failed to get response: {}", e))?;

    // La risposta è una stringa semplice, non un oggetto con tool_calls
    let current_response = response;
    
    if !current_response.is_empty() {
        current_messages.push(Message::Assistant {
            content: OneOrMany::one(AssistantContent::text(current_response)),
        });
    }

    Ok(current_messages)
}
