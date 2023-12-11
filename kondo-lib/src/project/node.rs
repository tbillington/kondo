use std::path::Path;

use super::Project;

#[derive(Debug, Clone, Copy)]
pub struct NodeProject;

impl Project for NodeProject {
    fn name(&self) -> &str {
        "Node"
    }

    fn is_project(&self, root_dir: &Path) -> bool {
        root_dir.join("package.json").exists()
    }

    fn is_artifact(&self, path: &Path) -> bool {
        path.is_dir() && path.file_name().is_some_and(|f| f == "node_modules")
    }
}
