use std::path::{Path, PathBuf};

use enum_dispatch::enum_dispatch;

pub mod cmake;
pub mod godot;
pub mod node;
pub mod rust;
pub mod unity;
pub mod unreal;

use cmake::CMakeProject;
use godot::GodotProject;
use node::NodeProject;
use rust::RustProject;
use unity::UnityProject;

#[enum_dispatch(ProjectEnum)]
pub trait Project {
    fn kind_name(&self) -> &'static str;
    fn is_project(&self, root_dir: &Path) -> bool;
    fn name(&self, root_dir: &Path) -> Option<String>;
    #[allow(unused_variables)]
    fn project_focus(&self, root_dir: &Path) -> Option<String> {
        None
    }
    /// Assumes `root_path` is a valid `Project` of the same kind resulting from `Project::is_proejct`
    fn is_root_artifact(&self, root_path: &Path) -> bool;
    /// Assumes `root_dir` is a valid `Project` of the same kind resulting from `Project::is_proejct`
    fn root_artifacts(&self, root_dir: &Path) -> Vec<PathBuf>;
}

#[enum_dispatch]
#[derive(Debug, Clone, Copy)]
pub enum ProjectEnum {
    CMakeProject,
    NodeProject,
    RustProject,
    UnityProject,
    GodotProject,
}

impl ProjectEnum {
    pub const ALL: &'static [Self] = &[
        Self::RustProject(RustProject),
        Self::NodeProject(NodeProject),
        Self::UnityProject(UnityProject),
        Self::CMakeProject(CMakeProject),
        Self::GodotProject(GodotProject),
    ];

    pub fn artifact_size(&self, path: &Path) -> u64 {
        self.root_artifacts(path).into_iter().map(dir_size).sum()
    }

    pub fn last_modified(&self, path: &Path) -> Result<std::time::SystemTime, std::io::Error> {
        let top_level_modified = std::fs::metadata(path)?.modified()?;
        let most_recent_modified = path
            .read_dir()?
            .flatten()
            .filter(|entry| !self.is_root_artifact(&entry.path()))
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

pub fn dir_size<P: AsRef<Path>>(path: P) -> u64 {
    fn dir_size_inner(path: &Path) -> u64 {
        walkdir::WalkDir::new(path)
            .same_file_system(true)
            .follow_root_links(false)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.metadata().map(|md| md.len()).ok())
            .sum()
    }
    dir_size_inner(path.as_ref())
}

fn filter_paths_exist<'a>(root: &'a Path, paths: &'a [&str]) -> impl Iterator<Item = PathBuf> + 'a {
    paths.iter().filter_map(|p| {
        let path = root.join(p);
        path.exists().then_some(path)
    })
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
