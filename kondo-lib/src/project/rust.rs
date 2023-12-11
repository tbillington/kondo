use std::path::Path;

use super::Project;

#[derive(Debug, Clone, Copy)]
pub struct RustProject;

impl Project for RustProject {
    fn name(&self) -> &str {
        "Rust"
    }

    fn is_project(&self, root_dir: &Path) -> bool {
        root_dir.join("Cargo.toml").exists()
    }

    fn is_artifact(&self, path: &Path) -> bool {
        path.is_dir() && path.file_name().is_some_and(|f| f == "target")
    }
}