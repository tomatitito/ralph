//! JSON event parsing for Claude's streaming output.
//!
//! Claude's `--output-format stream-json` produces JSONL with these event types:
//! - `init`: System initialization message
//! - `assistant`: Claude's response content
//! - `tool_use`: Tool call requests
//! - `tool_result`: Tool call results
//! - `result`: Final summary with token usage and cost

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{RalphError, Result};

/// Token usage statistics from a result event
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

/// A parsed Claude streaming event
#[derive(Debug, Clone)]
pub enum ClaudeEvent {
    /// System initialization
    Init { session_id: Option<String> },
    /// Assistant message content
    Assistant { content: Vec<ContentBlock> },
    /// Tool use request
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    /// Tool result
    ToolResult { id: String, content: String },
    /// Final result with usage statistics
    Result {
        session_id: Option<String>,
        usage: TokenUsage,
        total_cost_usd: Option<f64>,
    },
    /// Unknown event type (for forward compatibility)
    Unknown { event_type: String, raw: Value },
}

impl ClaudeEvent {
    /// Parse a JSON line into a ClaudeEvent
    pub fn parse(line: &str) -> Result<Self> {
        let line = line.trim();
        if line.is_empty() {
            return Err(RalphError::JsonParseError("Empty line".to_string()));
        }

        let value: Value =
            serde_json::from_str(line).map_err(|e| RalphError::JsonParseError(e.to_string()))?;

        let event_type = value
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("unknown");

        match event_type {
            "init" | "system" => Ok(ClaudeEvent::Init {
                session_id: value
                    .get("session_id")
                    .and_then(|s| s.as_str())
                    .map(String::from),
            }),
            "assistant" => {
                let content = if let Some(message) = value.get("message") {
                    // New format: { type: "assistant", message: { content: [...] } }
                    message
                        .get("content")
                        .and_then(|c| serde_json::from_value(c.clone()).ok())
                        .unwrap_or_default()
                } else if let Some(content) = value.get("content") {
                    // Direct content array
                    serde_json::from_value(content.clone()).unwrap_or_default()
                } else {
                    Vec::new()
                };
                Ok(ClaudeEvent::Assistant { content })
            }
            "tool_use" => {
                let id = value
                    .get("id")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = value
                    .get("name")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                let input = value.get("input").cloned().unwrap_or(Value::Null);
                Ok(ClaudeEvent::ToolUse { id, name, input })
            }
            "tool_result" => {
                let id = value
                    .get("tool_use_id")
                    .or_else(|| value.get("id"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                let content = value
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(ClaudeEvent::ToolResult { id, content })
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
                let total_cost_usd = value.get("total_cost_usd").and_then(|c| c.as_f64());
                Ok(ClaudeEvent::Result {
                    session_id,
                    usage,
                    total_cost_usd,
                })
            }
            _ => Ok(ClaudeEvent::Unknown {
                event_type: event_type.to_string(),
                raw: value,
            }),
        }
    }

    /// Extract plain text content from an assistant event
    pub fn extract_text(&self) -> Option<String> {
        match self {
            ClaudeEvent::Assistant { content } => {
                let texts: Vec<String> = content
                    .iter()
                    .filter_map(|block| match block {
                        ContentBlock::Text { text } => Some(text.clone()),
                        _ => None,
                    })
                    .collect();
                if texts.is_empty() {
                    None
                } else {
                    Some(texts.join("\n"))
                }
            }
            _ => None,
        }
    }

    /// Check if this event contains token usage info
    pub fn get_usage(&self) -> Option<&TokenUsage> {
        match self {
            ClaudeEvent::Result { usage, .. } => Some(usage),
            _ => None,
        }
    }

    /// Get the event type as a string for logging
    pub fn event_type(&self) -> &str {
        match self {
            ClaudeEvent::Init { .. } => "init",
            ClaudeEvent::Assistant { .. } => "assistant",
            ClaudeEvent::ToolUse { .. } => "tool_use",
            ClaudeEvent::ToolResult { .. } => "tool_result",
            ClaudeEvent::Result { .. } => "result",
            ClaudeEvent::Unknown { event_type, .. } => event_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_assistant_event() {
        let json = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Hello, world!"}]}}"#;
        let event = ClaudeEvent::parse(json).unwrap();

        if let ClaudeEvent::Assistant { content } = event {
            assert_eq!(content.len(), 1);
            if let ContentBlock::Text { text } = &content[0] {
                assert_eq!(text, "Hello, world!");
            } else {
                panic!("Expected text block");
            }
        } else {
            panic!("Expected assistant event");
        }
    }

    #[test]
    fn test_parse_tool_use_event() {
        let json =
            r#"{"type":"tool_use","id":"tool_123","name":"Read","input":{"file_path":"/test.rs"}}"#;
        let event = ClaudeEvent::parse(json).unwrap();

        if let ClaudeEvent::ToolUse { id, name, input } = event {
            assert_eq!(id, "tool_123");
            assert_eq!(name, "Read");
            assert_eq!(input["file_path"], "/test.rs");
        } else {
            panic!("Expected tool_use event");
        }
    }

    #[test]
    fn test_parse_result_event() {
        let json = r#"{"type":"result","session_id":"sess_123","usage":{"input_tokens":1000,"output_tokens":500},"total_cost_usd":0.05}"#;
        let event = ClaudeEvent::parse(json).unwrap();

        if let ClaudeEvent::Result {
            session_id,
            usage,
            total_cost_usd,
        } = event
        {
            assert_eq!(session_id, Some("sess_123".to_string()));
            assert_eq!(usage.input_tokens, 1000);
            assert_eq!(usage.output_tokens, 500);
            assert_eq!(usage.total(), 1500);
            assert_eq!(total_cost_usd, Some(0.05));
        } else {
            panic!("Expected result event");
        }
    }

    #[test]
    fn test_parse_unknown_event() {
        let json = r#"{"type":"future_event","data":"something"}"#;
        let event = ClaudeEvent::parse(json).unwrap();

        if let ClaudeEvent::Unknown { event_type, .. } = event {
            assert_eq!(event_type, "future_event");
        } else {
            panic!("Expected unknown event");
        }
    }

    #[test]
    fn test_extract_text() {
        let json = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Hello"},{"type":"text","text":"World"}]}}"#;
        let event = ClaudeEvent::parse(json).unwrap();

        let text = event.extract_text().unwrap();
        assert_eq!(text, "Hello\nWorld");
    }

    #[test]
    fn test_extract_text_no_content() {
        let json = r#"{"type":"result","usage":{}}"#;
        let event = ClaudeEvent::parse(json).unwrap();

        assert!(event.extract_text().is_none());
    }
}
