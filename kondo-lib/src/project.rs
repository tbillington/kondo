use core::arch;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

use enum_dispatch::enum_dispatch;

mod node;
mod rust;
mod unity;

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
}

#[enum_dispatch(ProjectEnum)]
pub trait Project {
    fn name(&self) -> &str;
    fn is_project(&self, root_dir: &Path) -> bool;
    fn is_artifact(&self, path: &Path) -> bool;
}

#[derive(Default)]
pub(crate) struct TestDirectoryBuilder {
    files: Vec<String>,
    artifacts: Vec<String>,
}

impl TestDirectoryBuilder {
    pub(crate) fn build(self) -> Result<TestDirectory, std::io::Error> {
        let root = std::env::temp_dir().join("kondo-test");
        let _ = std::fs::remove_dir_all(&root);

        let create_files = |files: Vec<String>| -> Result<Vec<PathBuf>, std::io::Error> {
            files
                .into_iter()
                .map(|f| -> Result<PathBuf, std::io::Error> {
                    let f = root.join(f);
                    std::fs::create_dir_all(f.parent().unwrap())?;
                    std::fs::File::create(&f)?.write_all(b"test")?;
                    Ok(f)
                })
                .collect::<Result<Vec<_>, _>>()
        };

        let files = create_files(self.files)?;

        let artifacts = create_files(self.artifacts)?;

        Ok(TestDirectory {
            root,
            files,
            artifacts,
        })
    }

    pub(crate) fn file(mut self, path: &str) -> Self {
        self.files.push(path.to_string());
        self
    }

    pub(crate) fn artifact(mut self, path: &str) -> Self {
        self.artifacts.push(path.to_string());
        self
    }
}

pub(crate) struct TestDirectory {
    root: PathBuf,
    files: Vec<PathBuf>,
    artifacts: Vec<PathBuf>,
}

impl TestDirectory {
    // pub(crate) fn artifact_bytes(&self) -> usize {
    //     self.artifacts
    //         .iter()
    //         .map(|p| std::fs::metadata(p).unwrap().len() as usize)
    //         .sum()
    // }

    pub(crate) fn fully_clean(&self) -> bool {
        self.artifacts.iter().all(|p| !p.exists())
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bramm() {
        let pp = TestDirectoryBuilder::default()
            .file("package.json")
            .file("src/main.js")
            .artifact("node_modules/foo")
            .artifact("node_modules/bar")
            .build()
            .unwrap();

        assert!(NodeProject.is_project(&pp.root));

        pp.artifacts.iter().for_each(|p| {
            std::fs::remove_file(p).unwrap();
        });

        // do clean

        assert!(pp.fully_clean());
        // assert!(node_p.artifact_bytes(), 20);
    }
}
