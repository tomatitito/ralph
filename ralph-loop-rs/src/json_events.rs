//! JSON event parsing for supported coding agent CLIs.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::AgentProvider;
use crate::error::{RalphError, Result};

/// Token usage statistics from an agent result event
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    #[serde(default)]
    pub input_tokens: usize,
    #[serde(default)]
    pub output_tokens: usize,
    #[serde(default)]
    pub cache_creation_input_tokens: usize,
    #[serde(default)]
    pub cache_read_input_tokens: usize,
    #[serde(default)]
    pub cached_input_tokens: usize,
}

impl TokenUsage {
    /// Get total tokens (input + output)
    pub fn total(&self) -> usize {
        self.input_tokens + self.output_tokens
    }
}

/// Content block within an assistant message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    #[serde(other)]
    Other,
}

/// A normalized parsed JSON event from a supported agent backend
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Session or thread start
    SessionStart { session_id: Option<String> },
    /// Assistant message content
    AssistantMessage { text: String },
    /// Final result with token usage statistics
    Result {
        session_id: Option<String>,
        usage: TokenUsage,
    },
    /// Unknown event type (for forward compatibility)
    Unknown { event_type: String, raw: Value },
}

impl AgentEvent {
    /// Parse a JSON line into a normalized agent event
    pub fn parse(provider: AgentProvider, line: &str) -> Result<Self> {
        let line = line.trim();
        if line.is_empty() {
            return Err(RalphError::JsonParseError("Empty line".to_string()));
        }

        let value: Value =
            serde_json::from_str(line).map_err(|e| RalphError::JsonParseError(e.to_string()))?;

        match provider {
            AgentProvider::Claude => parse_claude_event(value),
            AgentProvider::Codex => parse_codex_event(value),
        }
    }

    /// Extract plain text content from an assistant event
    pub fn extract_text(&self) -> Option<&str> {
        match self {
            AgentEvent::AssistantMessage { text } => Some(text),
            _ => None,
        }
    }

    /// Check if this event contains token usage info
    pub fn get_usage(&self) -> Option<&TokenUsage> {
        match self {
            AgentEvent::Result { usage, .. } => Some(usage),
            _ => None,
        }
    }

    /// Get the event type as a string for logging
    pub fn event_type(&self) -> &str {
        match self {
            AgentEvent::SessionStart { .. } => "session_start",
            AgentEvent::AssistantMessage { .. } => "assistant_message",
            AgentEvent::Result { .. } => "result",
            AgentEvent::Unknown { event_type, .. } => event_type,
        }
    }
}

fn parse_claude_event(value: Value) -> Result<AgentEvent> {
    let event_type = value
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    match event_type {
        "init" | "system" => Ok(AgentEvent::SessionStart {
            session_id: value
                .get("session_id")
                .and_then(|s| s.as_str())
                .map(String::from),
        }),
        "assistant" => {
            let content: Vec<ContentBlock> = if let Some(message) = value.get("message") {
                message
                    .get("content")
                    .and_then(|c| serde_json::from_value(c.clone()).ok())
                    .unwrap_or_default()
            } else if let Some(content) = value.get("content") {
                serde_json::from_value(content.clone()).unwrap_or_default()
            } else {
                Vec::new()
            };

            let text = content
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");

            Ok(AgentEvent::AssistantMessage { text })
        }
        "result" => {
            let session_id = value
                .get("session_id")
                .and_then(|s| s.as_str())
                .map(String::from);
            let usage = value
                .get("usage")
                .and_then(|u| serde_json::from_value(u.clone()).ok())
                .unwrap_or_default();
            Ok(AgentEvent::Result { session_id, usage })
        }
        _ => Ok(AgentEvent::Unknown {
            event_type: event_type.to_string(),
            raw: value,
        }),
    }
}

fn parse_codex_event(value: Value) -> Result<AgentEvent> {
    let event_type = value
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    match event_type {
        "thread.started" => Ok(AgentEvent::SessionStart {
            session_id: value
                .get("thread_id")
                .and_then(|s| s.as_str())
                .map(String::from),
        }),
        "item.completed" => {
            let item = value.get("item").cloned().unwrap_or(Value::Null);
            let item_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if item_type == "agent_message" {
                let text = item
                    .get("text")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(AgentEvent::AssistantMessage { text })
            } else {
                Ok(AgentEvent::Unknown {
                    event_type: event_type.to_string(),
                    raw: value,
                })
            }
        }
        "turn.completed" => {
            let usage = value
                .get("usage")
                .and_then(|u| serde_json::from_value(u.clone()).ok())
                .unwrap_or_default();
            Ok(AgentEvent::Result {
                session_id: None,
                usage,
            })
        }
        _ => Ok(AgentEvent::Unknown {
            event_type: event_type.to_string(),
            raw: value,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_claude_assistant_event() {
        let json = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Hello, world!"}]}}"#;
        let event = AgentEvent::parse(AgentProvider::Claude, json).unwrap();

        assert_eq!(event.extract_text(), Some("Hello, world!"));
    }

    #[test]
    fn test_parse_claude_result_event() {
        let json = r#"{"type":"result","session_id":"sess_123","usage":{"input_tokens":1000,"output_tokens":500},"total_cost_usd":0.05}"#;
        let event = AgentEvent::parse(AgentProvider::Claude, json).unwrap();

        if let AgentEvent::Result { session_id, usage } = event {
            assert_eq!(session_id, Some("sess_123".to_string()));
            assert_eq!(usage.input_tokens, 1000);
            assert_eq!(usage.output_tokens, 500);
            assert_eq!(usage.total(), 1500);
        } else {
            panic!("Expected result event");
        }
    }

    #[test]
    fn test_parse_codex_thread_started_event() {
        let json = r#"{"type":"thread.started","thread_id":"thread_123"}"#;
        let event = AgentEvent::parse(AgentProvider::Codex, json).unwrap();

        if let AgentEvent::SessionStart { session_id } = event {
            assert_eq!(session_id, Some("thread_123".to_string()));
        } else {
            panic!("Expected session_start event");
        }
    }

    #[test]
    fn test_parse_codex_agent_message_event() {
        let json = r#"{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"hello\nTASK COMPLETE"}}"#;
        let event = AgentEvent::parse(AgentProvider::Codex, json).unwrap();

        assert_eq!(event.extract_text(), Some("hello\nTASK COMPLETE"));
    }

    #[test]
    fn test_parse_codex_turn_completed_event() {
        let json = r#"{"type":"turn.completed","usage":{"input_tokens":17725,"cached_input_tokens":3456,"output_tokens":45}}"#;
        let event = AgentEvent::parse(AgentProvider::Codex, json).unwrap();

        if let AgentEvent::Result { session_id, usage } = event {
            assert_eq!(session_id, None);
            assert_eq!(usage.input_tokens, 17725);
            assert_eq!(usage.cached_input_tokens, 3456);
            assert_eq!(usage.output_tokens, 45);
            assert_eq!(usage.total(), 17770);
        } else {
            panic!("Expected result event");
        }
    }
}
