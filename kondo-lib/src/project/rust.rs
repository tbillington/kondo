use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::project::utils::filter_exists;

use super::Project;

#[derive(Debug, Clone, Copy)]
pub struct RustProject;

impl Project for RustProject {
    fn kind_name(&self) -> &'static str {
        "Rust"
    }

    fn name(&self, root_dir: &Path) -> Option<String> {
        toml::from_str::<CargoToml>(&std::fs::read_to_string(root_dir.join("Cargo.toml")).ok()?)
            .ok()?
            .package?
            .name
    }

    fn is_project(&self, root_dir: &Path) -> bool {
        root_dir.join("Cargo.toml").exists()
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

const PATHS: [&str; 2] = ["target", ".xwin-cache"];

#[derive(Deserialize)]
struct CargoToml {
    package: Option<CargoTomlPackage>,
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
