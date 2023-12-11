use std::path::Path;

use super::Project;

#[derive(Debug, Clone, Copy)]
pub struct UnityProject;

impl Project for UnityProject {
    fn name(&self) -> &str {
        "Unity"
    }

    fn is_project(&self, root_dir: &Path) -> bool {
        root_dir.join("Assembly-CSharp.csproj").exists()
    }

    fn is_artifact(&self, path: &Path) -> bool {
        path.is_dir() && path.file_name().is_some_and(|f| f == "Library")
    }
}
