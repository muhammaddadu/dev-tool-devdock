//! Optional local AI CLI integration. Manual, never run in the background.
//! M7 implementation.

pub struct AiEngine {}

impl AiEngine {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AiEngine {
    fn default() -> Self {
        Self::new()
    }
}
