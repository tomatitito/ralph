//! Transcript file parsing.
//!
//! Parses JSONL transcript files from ralph-loop.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;

/// Token usage statistics
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

/// A parsed transcript event
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TranscriptEvent {
    /// System initialization
    Init { session_id: Option<String> },
    /// User message
    User { content: String },
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
    /// Unknown event type
    Unknown { event_type: String, raw: Value },
}

impl TranscriptEvent {
    /// Parse a JSON line into a TranscriptEvent
    pub fn parse(line: &str) -> Option<Self> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        let value: Value = serde_json::from_str(line).ok()?;
        let event_type = value
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("unknown");

        Some(match event_type {
            "init" | "system" => TranscriptEvent::Init {
                session_id: value
                    .get("session_id")
                    .and_then(|s| s.as_str())
                    .map(String::from),
            },
            "user" => {
                // User messages can have content as a string or array
                let content = if let Some(message) = value.get("message") {
                    if let Some(content) = message.get("content") {
                        if content.is_string() {
                            content.as_str().unwrap_or("").to_string()
                        } else if content.is_array() {
                            // Extract text from content array
                            content
                                .as_array()
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|item| {
                                            if item.get("type").and_then(|t| t.as_str())
                                                == Some("text")
                                            {
                                                item.get("text").and_then(|t| t.as_str())
                                            } else {
                                                None
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                })
                                .unwrap_or_default()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                TranscriptEvent::User { content }
            }
            "assistant" => {
                let content = if let Some(message) = value.get("message") {
                    message
                        .get("content")
                        .and_then(|c| serde_json::from_value(c.clone()).ok())
                        .unwrap_or_default()
                } else if let Some(content) = value.get("content") {
                    serde_json::from_value(content.clone()).unwrap_or_default()
                } else {
                    Vec::new()
                };
                TranscriptEvent::Assistant { content }
            }
            "tool_use" => TranscriptEvent::ToolUse {
                id: value
                    .get("id")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
                name: value
                    .get("name")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
                input: value.get("input").cloned().unwrap_or(Value::Null),
            },
            "tool_result" => TranscriptEvent::ToolResult {
                id: value
                    .get("tool_use_id")
                    .or_else(|| value.get("id"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
                content: value
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string(),
            },
            "result" => TranscriptEvent::Result {
                session_id: value
                    .get("session_id")
                    .and_then(|s| s.as_str())
                    .map(String::from),
                usage: value
                    .get("usage")
                    .and_then(|u| serde_json::from_value(u.clone()).ok())
                    .unwrap_or_default(),
                total_cost_usd: value.get("total_cost_usd").and_then(|c| c.as_f64()),
            },
            _ => TranscriptEvent::Unknown {
                event_type: event_type.to_string(),
                raw: value,
            },
        })
    }

    /// Extract plain text content from an assistant event
    #[allow(dead_code)]
    pub fn extract_text(&self) -> Option<String> {
        match self {
            TranscriptEvent::Assistant { content } => {
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
}

/// Read and parse all events from a transcript file
pub fn read_transcript(path: &Path) -> Result<Vec<TranscriptEvent>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Some(event) = TranscriptEvent::parse(&line) {
            events.push(event);
        }
    }

    Ok(events)
}

/// Read events from a transcript file starting at a specific line
#[allow(dead_code)]
pub fn read_transcript_from_line(path: &Path, start_line: usize) -> Result<Vec<TranscriptEvent>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for (i, line) in reader.lines().enumerate() {
        if i < start_line {
            continue;
        }
        let line = line?;
        if let Some(event) = TranscriptEvent::parse(&line) {
            events.push(event);
        }
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_assistant_event() {
        let json =
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Hello!"}]}}"#;
        let event = TranscriptEvent::parse(json).unwrap();

        if let TranscriptEvent::Assistant { content } = event {
            assert_eq!(content.len(), 1);
        } else {
            panic!("Expected assistant event");
        }
    }

    #[test]
    fn test_parse_result_event() {
        let json = r#"{"type":"result","usage":{"input_tokens":100,"output_tokens":50}}"#;
        let event = TranscriptEvent::parse(json).unwrap();

        if let TranscriptEvent::Result { usage, .. } = event {
            assert_eq!(usage.total(), 150);
        } else {
            panic!("Expected result event");
        }
    }

    #[test]
    fn test_extract_text() {
        let json =
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Hello world"}]}}"#;
        let event = TranscriptEvent::parse(json).unwrap();

        assert_eq!(event.extract_text(), Some("Hello world".to_string()));
    }
}
