use std::path::{Path, PathBuf};

use super::{filter_paths_exist, Project};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerraformProject;

const ROOT_ARTIFACT_PATHS: &[&str] = &[".terraform"];

impl Project for TerraformProject {
    fn kind_name(&self) -> &'static str {
        "Terraform"
    }

    fn name(&self, root_dir: &Path) -> Option<String> {
        None
    }

    fn is_project(&self, root_dir: &Path) -> bool {
        let terraform_hcl = root_dir.join(".terraform.lock.hcl");

        if !terraform_hcl.exists() || !terraform_hcl.is_file() {
            return false;
        }

        return true;
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
    fn terraform_project_minamal() {
        let td = TestDirectoryBuilder::default()
            .file(".terraform.lock.hcl")
            .build()
            .unwrap();

        assert!(TerraformProject.is_project(&td.root));
    }

    #[test]
    fn terraform_project_typical() {
        let td = TestDirectoryBuilder::default()
            .file(".terraform.lock.hcl")
            .file("main.tf")
            .artifact(".terraform/providers")
            .build()
            .unwrap();

        assert!(TerraformProject.is_project(&td.root));
    }
}
