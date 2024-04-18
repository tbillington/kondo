use std::path::{Path, PathBuf};

use enum_dispatch::enum_dispatch;

mod node;
mod rust;
mod unity;
mod utils;

use node::NodeProject;
use rust::RustProject;
use unity::UnityProject;

#[enum_dispatch]
#[derive(Debug, Clone, Copy)]
pub enum ProjectEnum {
    RustProject,
    NodeProject,
    UnityProject,
}

impl ProjectEnum {
    pub const ALL: [Self; 3] = [
        Self::RustProject(RustProject),
        Self::NodeProject(NodeProject),
        Self::UnityProject(UnityProject),
    ];

    pub fn artifact_size(&self, path: &Path) -> u64 {
        self.artifacts(path)
            .into_iter()
            .map(|path| {
                walkdir::WalkDir::new(path)
                    .same_file_system(true)
                    .follow_root_links(false)
                    .follow_links(false)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file())
                    .filter_map(|e| e.metadata().map(|md| md.len()).ok())
                    .sum::<u64>()
            })
            .sum()
    }

    pub fn last_modified(&self, path: &Path) -> Result<std::time::SystemTime, std::io::Error> {
        let top_level_modified = std::fs::metadata(path)?.modified()?;
        let most_recent_modified = path
            .read_dir()?
            .flatten()
            .filter(|entry| !self.is_artifact(&entry.path()))
            .filter_map(|entry| {
                let file_type = entry.file_type().ok()?;
                Some((entry, file_type))
            })
            .fold(top_level_modified, |acc, (entry, file_type)| {
                if file_type.is_file() {
                    if let Ok(e) = entry.metadata() {
                        if let Ok(modified) = e.modified() {
                            if modified > acc {
                                return modified;
                            }
                        }
                    }
                } else if file_type.is_file() {
                    return walkdir::WalkDir::new(path)
                        .same_file_system(true)
                        .follow_root_links(false)
                        .follow_links(false)
                        .into_iter()
                        .filter_map(Result::ok)
                        .filter_map(|e| e.metadata().ok()?.modified().ok())
                        .fold(
                            acc,
                            |acc, last_mod| {
                                if last_mod > acc {
                                    last_mod
                                } else {
                                    acc
                                }
                            },
                        );
                }
                acc
            });

        Ok(most_recent_modified)
    }
}

#[enum_dispatch(ProjectEnum)]
pub trait Project {
    fn kind_name(&self) -> &'static str;
    fn name(&self, root_dir: &Path) -> Option<String>;
    fn is_project(&self, root_dir: &Path) -> bool;
    fn is_artifact(&self, path: &Path) -> bool;
    fn artifacts(&self, root_dir: &Path) -> Vec<PathBuf>;
}

// #[cfg(test)]
// mod tests {
//     use crate::test::TestDirectoryBuilder;

//     use super::*;

//     #[test]
//     fn bramm() {
//         let td = TestDirectoryBuilder::default()
//             .file("package.json")
//             .file("src/main.js")
//             .artifact("node_modules/foo")
//             .artifact("node_modules/bar")
//             .build()
//             .unwrap();

//         assert!(NodeProject.is_project(&pp.root));

//         pp.artifacts.iter().for_each(|p| {
//             std::fs::remove_file(p).unwrap();
//         });

//         // do clean

//         assert!(pp.fully_clean());
//         // assert!(node_p.artifact_bytes(), 20);
//     }
// }
