//! Provider clients and shared AI transport types for Tau.
//!
//! Defines request/response schemas, model/provider abstractions, and retry
//! behavior used by OpenAI-, Anthropic-, and Google-compatible backends.

mod anthropic;
mod google;
mod openai;
mod provider;
mod retry;
mod textual_tool_calls;
mod types;

pub use anthropic::{AnthropicClient, AnthropicConfig};
pub use google::{GoogleClient, GoogleConfig};
pub use openai::{OpenAiAuthScheme, OpenAiClient, OpenAiConfig};
pub use provider::{ModelRef, ModelRefParseError, Provider};
pub use textual_tool_calls::promote_assistant_textual_tool_calls;
pub use types::{
    ChatRequest, ChatResponse, ChatUsage, ContentBlock, LlmClient, MediaSource, Message,
    MessageRole, PromptCacheConfig, ProviderErrorKind, StreamDeltaHandler, TauAiError, ToolCall,
    ToolChoice, ToolDefinition,
};
