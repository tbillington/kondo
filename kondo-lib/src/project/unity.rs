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

#[cfg(test)]
mod tests {
    use crate::test::TestDirectoryBuilder;

    use super::*;

    #[test]
    fn unity_project_minimal() {
        let pp = TestDirectoryBuilder::default()
            .file("Assembly-CSharp.csproj")
            .build()
            .unwrap();

        assert!(UnityProject.is_project(&pp.root));
    }

    #[test]
    fn unity_project_typical() {
        let pp = TestDirectoryBuilder::default()
            .file("Assembly-CSharp.csproj")
            .file("Assembly-CSharp-Editor.csproj")
            .file("FunGame.sln")
            .file("Assets/script.cs")
            .artifact("Library/foo")
            .build()
            .unwrap();

        assert!(UnityProject.is_project(&pp.root));
    }
}
