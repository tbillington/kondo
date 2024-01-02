use std::path::{Path, PathBuf};

use crate::project::utils::filter_exists;

use super::Project;

#[derive(Debug, Clone, Copy)]
pub struct UnityProject;

impl Project for UnityProject {
    fn kind_name(&self) -> &str {
        "Unity"
    }

    fn name(&self, root_dir: &Path) -> Option<String> {
        unity::project_name(root_dir).ok().flatten()
    }

    fn is_project(&self, root_dir: &Path) -> bool {
        root_dir.join("Assembly-CSharp.csproj").exists()
    }

    fn is_artifact(&self, path: &Path) -> bool {
        path.is_dir()
            && path
                .file_name()
                .is_some_and(|f| PATHS.iter().any(|p| *p == f))
    }

    fn artifacts(&self, root_dir: &Path) -> Vec<PathBuf> {
        filter_exists(root_dir, &PATHS).collect()
    }
}

const PATHS: [&str; 7] = [
    "Library",
    "Temp",
    "Obj",
    "Logs",
    "MemoryCaptures",
    "Build",
    "Builds",
];

#[cfg(test)]
mod tests {
    use crate::test::TestDirectoryBuilder;

    use super::*;

    #[test]
    fn unity_project_minimal() {
        let td = TestDirectoryBuilder::default()
            .file("Assembly-CSharp.csproj")
            .build()
            .unwrap();

        assert!(UnityProject.is_project(&td.root));
    }

    #[test]
    fn unity_project_typical() {
        let td = TestDirectoryBuilder::default()
            .file("Assembly-CSharp.csproj")
            .file("Assembly-CSharp-Editor.csproj")
            .file("FunGame.sln")
            .file("Assets/script.cs")
            .artifact("Library/foo")
            .build()
            .unwrap();

        assert!(UnityProject.is_project(&td.root));
    }

    #[test]
    fn unity_project_name() {
        let td = TestDirectoryBuilder::default()
            .file("Assembly-CSharp.csproj")
            .file_content(
                "ProjectSettings/ProjectSettings.asset",
                r#"
%YAML 1.1
PlayerSettings:
  productName: PossumGame
                "#,
            )
            .build()
            .unwrap();

        assert_eq!(UnityProject.name(&td.root), Some("PossumGame".to_string()));
    }
}
