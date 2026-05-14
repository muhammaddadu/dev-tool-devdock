use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AiProviderId {
    Claude,
    Codex,
    Cursor,
    Ollama,
}

impl AiProviderId {
    pub fn display_name(&self) -> &'static str {
        match self {
            AiProviderId::Claude => "Claude",
            AiProviderId::Codex => "Codex",
            AiProviderId::Cursor => "Cursor",
            AiProviderId::Ollama => "Ollama",
        }
    }

    pub fn cli_binary(&self) -> &'static str {
        match self {
            AiProviderId::Claude => "claude",
            AiProviderId::Codex => "codex",
            AiProviderId::Cursor => "cursor",
            AiProviderId::Ollama => "ollama",
        }
    }
}
