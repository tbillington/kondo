use std::path::{Path, PathBuf};

use super::{filter_paths_exist, Project};

#[derive(Debug, Clone, Copy)]
pub struct TemplateProject;

const ROOT_ARTIFACT_PATHS: &[&str] = &[
    "Binaries",
    "Build",
    "Saved",
    "DerivedDataCache",
    "Intermediate",
];

impl Project for TemplateProject {
    fn kind_name(&self) -> &'static str {
        "Unreal"
    }

    fn name(&self, _root_dir: &Path) -> Option<String> {
        // TODO
        None
    }

    fn is_project(&self, root_dir: &Path) -> bool {
        root_dir.join(".uproject").exists()
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
    fn unreal_project_minimal() {
        let td = TestDirectoryBuilder::default()
            .file(".uproject")
            .build()
            .unwrap();

        assert!(TemplateProject.is_project(&td.root));
    }

    #[test]
    fn unreal_project_typical() {
        let td = TestDirectoryBuilder::default()
            .file(".uproject")
            .build()
            .unwrap();

        assert!(TemplateProject.is_project(&td.root));
    }

    #[test]
    fn unreal_project_name() {
        let td = TestDirectoryBuilder::default()
            .file(".uproject")
            .build()
            .unwrap();

        assert_eq!(TemplateProject.name(&td.root), None);
    }
}
