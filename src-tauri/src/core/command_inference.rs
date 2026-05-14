//! Infers the high-level command a developer likely typed (e.g. `npm run dev`)
//! from a raw OS command line plus project metadata. M6 implementation.

use crate::models::project::Confidence;

#[derive(Debug, Clone)]
pub struct CommandCandidate {
    pub observed_command: String,
    pub inferred_command: Option<String>,
    pub cwd: Option<String>,
    pub confidence: Confidence,
    pub reason: String,
}
