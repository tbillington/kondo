use std::path::{Path, PathBuf};

use crate::project::utils::filter_paths_exist;

use super::Project;

#[derive(Debug, Clone, Copy)]
pub struct UnityProject;

const ROOT_ARTIFACT_PATHS: &[&str] = &[
    "Library",
    "Temp",
    "Obj",
    "Logs",
    "MemoryCaptures",
    "Build",
    "Builds",
];

impl Project for UnityProject {
    fn kind_name(&self) -> &'static str {
        "Unity"
    }

    fn name(&self, root_dir: &Path) -> Option<String> {
        unity::project_name(root_dir).ok().flatten()
    }

    fn is_project(&self, root_dir: &Path) -> bool {
        root_dir.join("Assembly-CSharp.csproj").exists()
    }

    fn is_root_artifact(&self, root_path: &Path) -> bool {
        root_path.is_dir()
            && root_path
                .file_name()
                .is_some_and(|f| ROOT_ARTIFACT_PATHS.iter().any(|p| *p == f))
    }

    fn root_artifacts(&self, root_dir: &Path) -> Vec<PathBuf> {
        filter_paths_exist(root_dir, ROOT_ARTIFACT_PATHS).collect()
    }
}

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
