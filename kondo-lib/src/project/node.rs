use std::path::Path;

use miniserde::Deserialize;

use super::Project;

#[derive(Debug, Clone, Copy)]
pub struct NodeProject;

impl Project for NodeProject {
    fn name(&self) -> &str {
        "Node"
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
            miniserde::json::from_str::<PackageManifestUnity>(&package_json_contents)
        else {
            return true;
        };

        match package_json_contents.unity {
            None => true,
            Some(unity_version) => unity_version.is_empty(),
        }
    }

    fn is_artifact(&self, path: &Path) -> bool {
        path.is_dir() && path.file_name().is_some_and(|f| f == "node_modules")
    }
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
        let pp = TestDirectoryBuilder::default()
            .file("package.json")
            .build()
            .unwrap();

        assert!(NodeProject.is_project(&pp.root));
    }

    #[test]
    fn node_typical() {
        let pp = TestDirectoryBuilder::default()
            .file("package.json")
            .file("index.js")
            .artifact("node_modules/index.js")
            .build()
            .unwrap();

        assert!(NodeProject.is_project(&pp.root));
    }

    #[test]
    fn ignore_unity_packages() {
        let pp = TestDirectoryBuilder::default()
            .file_content("package.json", r#"{"unity":"2019.4"}"#)
            .build()
            .unwrap();

        assert!(!NodeProject.is_project(&pp.root));
    }
}
