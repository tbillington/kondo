use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use super::{filter_paths_exist, Project};

#[derive(Debug, Clone, Copy)]
pub struct RustProject;

const ROOT_ARTIFACT_PATHS: &[&str] = &["target", ".xwin-cache"];

impl Project for RustProject {
    fn kind_name(&self) -> &'static str {
        "Rust"
    }

    fn is_project(&self, root_dir: &Path) -> bool {
        root_dir.join("Cargo.toml").exists()
    }

    fn name(&self, root_dir: &Path) -> Option<String> {
        let manifest = RustProject::manifest(root_dir)?;

        manifest.package?.name
    }

    fn project_focus(&self, root_dir: &Path) -> Option<String> {
        let manifest = RustProject::manifest(root_dir)?;

        let bevy = manifest
            .dependencies
            .is_some_and(|deps| deps.contains_key("bevy"))
            || manifest
                .workspace
                .is_some_and(|w| w.dependencies.is_some_and(|deps| deps.contains_key("bevy")));

        if bevy {
            Some("Bevy".to_string())
        } else {
            None
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

impl RustProject {
    fn manifest(root_dir: &Path) -> Option<CargoToml> {
        toml::from_str::<CargoToml>(&std::fs::read_to_string(root_dir.join("Cargo.toml")).ok()?)
            .ok()
    }
}

#[derive(Deserialize)]
struct CargoToml {
    package: Option<CargoTomlPackage>,
    dependencies: Option<
        HashMap<
            cargo_util_schemas::manifest::PackageName,
            cargo_util_schemas::manifest::InheritableDependency,
        >,
    >,
    workspace: Option<cargo_util_schemas::manifest::TomlWorkspace>,
}

#[derive(Deserialize)]
struct CargoTomlPackage {
    name: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::test::TestDirectoryBuilder;

    use super::*;

    #[test]
    fn rust_project_minimal() {
        let td = TestDirectoryBuilder::default()
            .file("Cargo.toml")
            .build()
            .unwrap();

        assert!(RustProject.is_project(&td.root));
    }

    #[test]
    fn rust_project_typical() {
        let td = TestDirectoryBuilder::default()
            .file("Cargo.toml")
            .file("src/main.rs")
            .artifact("target/proj")
            .build()
            .unwrap();

        assert!(RustProject.is_project(&td.root));
    }

    #[test]
    fn rust_project_name() {
        let td = TestDirectoryBuilder::default()
            .file_content(
                "Cargo.toml",
                r#"
[package]
name = "kondo"
                "#,
            )
            .build()
            .unwrap();

        assert_eq!(RustProject.name(&td.root), Some("kondo".to_string()));
    }
}
