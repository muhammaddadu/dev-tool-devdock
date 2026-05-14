//! Walks a project directory and produces a list of likely start commands by
//! inspecting `package.json`, `pyproject.toml`, `Gemfile`, etc. M6 implementation.

pub struct ProjectScanner {}

impl ProjectScanner {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ProjectScanner {
    fn default() -> Self {
        Self::new()
    }
}
