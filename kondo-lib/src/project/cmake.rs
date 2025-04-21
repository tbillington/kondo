//! Minimal CMake config.
//!
//! ```sh
//! touch CMakeLists.txt
//! ```
//!
//! All of these are valid ways to configure a CMake project, even simultaneously.
//!
//! ```sh
//! (mkdir build && cd build && cmake ..)                    # 1. Currently supported.
//! (mkdir _ && cd _ && cmake ..)                            # 2. Unsupported.
//! (mkdir build/config1 && cd build/config1 && cmake ../..) # 3. Unsupported.
//! (mkdir /tmp/build && cd /tmp/build && cmake /project/)   # 4. Unsupported, assumes project tree in `/project/`.
//! cmake .                                                  # 5. Unsupported.
//! ``````

use std::path::{Path, PathBuf};

use cmake_parser::{parse_cmakelists, Command, Doc};

use super::{filter_paths_exist, Project};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CMakeProject;

const ROOT_ARTIFACT_PATHS: [&str; 3] = ["build", "cmake-build-debug", "cmake-build-release"];

impl Project for CMakeProject {
    fn kind_name(&self) -> &'static str {
        "CMake"
    }

    fn name(&self, _root_dir: &Path) -> Option<String> {
        let cmakelists_data = std::fs::read(_root_dir.join("CMakeLists.txt")).ok()?;
        let cmake_doc = Doc::from(parse_cmakelists(&cmakelists_data).ok()?);

        let name = cmake_doc
            .to_commands_iter()
            .filter_map(Result::ok)
            .find_map(|cmd| {
                if let Command::Project(ph) = cmd {
                    Some(format!("{}", ph.project_name))
                } else {
                    None
                }
            });

        name
    }

    fn is_project(&self, root_dir: &Path) -> bool {
        root_dir.join("CMakeLists.txt").exists()
    }

    fn is_root_artifact(&self, root_path: &Path) -> bool {
        root_path.is_dir()
            && root_path
                .file_name()
                .is_some_and(|f| ROOT_ARTIFACT_PATHS.iter().any(|p| *p == f))
    }

    fn root_artifacts(&self, root_dir: &Path) -> Vec<PathBuf> {
        filter_paths_exist(root_dir, &ROOT_ARTIFACT_PATHS).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::test::TestDirectoryBuilder;

    use super::*;

    #[test]
    fn cmake_project_minimal() {
        let td = TestDirectoryBuilder::default()
            .file("CMakeLists.txt")
            .build()
            .unwrap();

        assert!(CMakeProject.is_project(&td.root));
    }

    #[test]
    fn cmake_project_name() {
        let proj_name = "foobar";

        let td = TestDirectoryBuilder::default()
            .file_content("CMakeLists.txt", &format!("project({proj_name})\n"))
            .build()
            .unwrap();

        assert_eq!(CMakeProject.name(&td.root), Some(proj_name.to_string()));
    }
}
