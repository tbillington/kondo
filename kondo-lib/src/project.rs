use std::path::Path;

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
