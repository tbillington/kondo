use std::path::{Path, PathBuf};

use super::{utils::filter_paths_exist, Project};

#[derive(Debug, Clone, Copy)]
pub struct GodotProject;

const ROOT_ARTIFACT_PATHS: &[&str] = &[".godot"];

impl Project for GodotProject {
    fn kind_name(&self) -> &'static str {
        "Godot"
    }

    fn name(&self, root_dir: &Path) -> Option<String> {
        ini::Ini::load_from_str(&std::fs::read_to_string(root_dir.join("project.godot")).ok()?)
            .ok()?
            .section(Some("application"))?
            .get("config/name")
            .map(ToString::to_string)
    }

    fn is_project(&self, root_dir: &Path) -> bool {
        root_dir.join("project.godot").exists()
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
    fn godot_project_minimal() {
        let td = TestDirectoryBuilder::default()
            .file("project.godot")
            .build()
            .unwrap();

        assert!(GodotProject.is_project(&td.root));
    }

    #[test]
    fn godot_project_typical() {
        let td = TestDirectoryBuilder::default()
            .file("project.godot")
            .file("Main.tscn")
            .artifact(".godot/blah")
            .build()
            .unwrap();

        assert!(GodotProject.is_project(&td.root));
    }

    #[test]
    fn godot_project_name() {
        let td = TestDirectoryBuilder::default()
            .file_content(
                "project.godot",
                r#"
[application]
config/name="PossumPossumOpossum"
                "#,
            )
            .build()
            .unwrap();

        assert_eq!(
            GodotProject.name(&td.root),
            Some("PossumPossumOpossum".to_string())
        );
    }
}
