use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{filter_paths_exist, Project};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeProject;

const ROOT_ARTIFACT_PATHS: &[&str] = &["node_modules", ".angular"];

impl Project for NodeProject {
    fn kind_name(&self) -> &'static str {
        "Node"
    }

    fn name(&self, root_dir: &Path) -> Option<String> {
        serde_json::from_str::<PackageJson>(
            &std::fs::read_to_string(root_dir.join("package.json")).ok()?,
        )
        .ok()?
        .name
    }

    fn is_project(&self, root_dir: &Path) -> bool {
        let package_json_path = root_dir.join("package.json");

        if !package_json_path.exists() || !package_json_path.is_file() {
            return false;
        }

        // check if it's a Unity package

        let Ok(package_json_contents) = std::fs::read_to_string(&package_json_path) else {
            return true;
        };

        let Ok(package_json_contents) =
            serde_json::from_str::<PackageManifestUnity>(&package_json_contents)
        else {
            return true;
        };

        match package_json_contents.unity {
            None => true,
            Some(unity_version) => unity_version.is_empty(),
        }
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

#[derive(Deserialize)]
struct PackageJson {
    name: Option<String>,
}

#[derive(Deserialize)]
struct PackageManifestUnity {
    unity: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::test::TestDirectoryBuilder;

    use super::*;

    #[test]
    fn node_minimal() {
        let td = TestDirectoryBuilder::default()
            .file("package.json")
            .build()
            .unwrap();

        assert!(NodeProject.is_project(&td.root));
    }

    #[test]
    fn node_typical() {
        let td = TestDirectoryBuilder::default()
            .file("package.json")
            .file("index.js")
            .artifact("node_modules/index.js")
            .build()
            .unwrap();

        assert!(NodeProject.is_project(&td.root));
    }

    #[test]
    fn node_project_name() {
        let td = TestDirectoryBuilder::default()
            .file_content("package.json", r#"{"name":"react"}"#)
            .build()
            .unwrap();

        assert_eq!(NodeProject.name(&td.root), Some("react".to_string()));
    }

    #[test]
    fn ignore_unity_packages() {
        let td = TestDirectoryBuilder::default()
            .file_content("package.json", r#"{"unity":"2019.4"}"#)
            .build()
            .unwrap();

        assert!(!NodeProject.is_project(&td.root));
    }
}
