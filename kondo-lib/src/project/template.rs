use std::path::{Path, PathBuf};

use super::{filter_paths_exist, Project};

#[derive(Debug, Clone, Copy)]
pub struct TemplateProject;

const ROOT_ARTIFACT_PATHS: &[&str] = &[];

impl Project for TemplateProject {
    fn kind_name(&self) -> &'static str {
        "Template"
    }

    fn name(&self, root_dir: &Path) -> Option<String> {
        None
    }

    fn is_project(&self, root_dir: &Path) -> bool {
        false
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
    fn template_project_minimal() {
        let td = TestDirectoryBuilder::default().build().unwrap();

        assert!(TemplateProject.is_project(&td.root));
    }

    #[test]
    fn template_project_typical() {
        let td = TestDirectoryBuilder::default().build().unwrap();

        assert!(TemplateProject.is_project(&td.root));
    }

    #[test]
    fn template_project_name() {
        let td = TestDirectoryBuilder::default().build().unwrap();

        assert_eq!(TemplateProject.name(&td.root), Some("Template".to_string()));
    }
}
