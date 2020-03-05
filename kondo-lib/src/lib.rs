use walkdir;

use std::path;

const SYMLINK_FOLLOW: bool = true;

const FILE_CARGO_TOML: &str = "Cargo.toml";
const FILE_PACKAGE_JSON: &str = "package.json";
const FILE_ASSEMBLY_CSHARP: &str = "Assembly-CSharp.csproj";
const FILE_STACK_HASKELL: &str = "stack.yaml";
const FILE_SBT_BUILD: &str = "build.sbt";
const FILE_MVN_BUILD: &str = "pom.xml";

const PROJECT_CARGO_DIRS: [&str; 1] = ["target"];
const PROJECT_NODE_DIRS: [&str; 1] = ["node_modules"];
const PROJECT_UNITY_DIRS: [&str; 7] = [
    "Library",
    "Temp",
    "Obj",
    "Logs",
    "MemoryCaptures",
    "Build",
    "Builds",
];
const PROJECT_STACK_DIRS: [&str; 1] = [".stack-work"];
const PROJECT_SBT_DIRS: [&str; 2] = ["target", "project/target"];
const PROJECT_MVN_DIRS: [&str; 1] = ["target"];

const PROJECT_CARGO_NAME: &str = "Cargo";
const PROJECT_NODE_NAME: &str = "Node";
const PROJECT_UNITY_NAME: &str = "Unity";
const PROJECT_STACK_NAME: &str = "Stack";
const PROJECT_SBT_NAME: &str = "SBT";
const PROJECT_MVN_NAME: &str = "Maven";

#[derive(Debug, Clone)]
pub enum ProjectType {
    Cargo,
    Node,
    Unity,
    Stack,
    SBT,
    Maven,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub project_type: ProjectType,
    pub path: path::PathBuf,
}

impl Project {
    pub fn artifact_dirs(&self) -> impl Iterator<Item = &&str> {
        match self.project_type {
            ProjectType::Cargo => PROJECT_CARGO_DIRS.iter(),
            ProjectType::Node => PROJECT_NODE_DIRS.iter(),
            ProjectType::Unity => PROJECT_UNITY_DIRS.iter(),
            ProjectType::Stack => PROJECT_STACK_DIRS.iter(),
            ProjectType::SBT => PROJECT_SBT_DIRS.iter(),
            ProjectType::Maven => PROJECT_MVN_DIRS.iter(),
        }
    }

    pub fn name(&self) -> String {
        self.path.to_str().unwrap().to_string()
    }

    pub fn size(&self) -> u64 {
        self.artifact_dirs()
            .map(|p| dir_size(&self.path.join(p)))
            .sum()
    }

    pub fn type_name(&self) -> &str {
        match self.project_type {
            ProjectType::Cargo => PROJECT_CARGO_NAME,
            ProjectType::Node => PROJECT_NODE_NAME,
            ProjectType::Unity => PROJECT_UNITY_NAME,
            ProjectType::Stack => PROJECT_STACK_NAME,
            ProjectType::SBT => PROJECT_SBT_NAME,
            ProjectType::Maven => PROJECT_MVN_NAME,
        }
    }
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

struct ProjectIter {
    it: walkdir::IntoIter,
}

impl Iterator for ProjectIter {
    type Item = Project;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry: walkdir::DirEntry = match self.it.next() {
                None => return None,
                Some(Err(_)) => continue,
                Some(Ok(entry)) => entry,
            };
            if !entry.file_type().is_dir() {
                continue;
            }
            if is_hidden(&entry) {
                self.it.skip_current_dir();
                continue;
            }
            let rd = match entry.path().read_dir() {
                Err(_) => continue,
                Ok(rd) => rd,
            };
            for dir_entry in rd.filter_map(|rd| rd.ok()) {
                let file_name = match dir_entry.file_name().into_string() {
                    Err(_) => continue,
                    Ok(file_name) => file_name,
                };
                let p_type = match file_name.as_str() {
                    FILE_CARGO_TOML => Some(ProjectType::Cargo),
                    FILE_PACKAGE_JSON => Some(ProjectType::Node),
                    FILE_ASSEMBLY_CSHARP => Some(ProjectType::Unity),
                    FILE_STACK_HASKELL => Some(ProjectType::Stack),
                    FILE_SBT_BUILD => Some(ProjectType::SBT),
                    FILE_MVN_BUILD => Some(ProjectType::Maven),
                    _ => None,
                };
                if let Some(project_type) = p_type {
                    self.it.skip_current_dir();
                    return Some(Project {
                        project_type,
                        path: entry.path().to_path_buf(),
                    });
                }
            }
        }
    }
}

pub fn scan<P: AsRef<path::Path>>(p: &P) -> impl Iterator<Item = Project> {
    ProjectIter {
        it: walkdir::WalkDir::new(p)
            .follow_links(SYMLINK_FOLLOW)
            .into_iter(),
    }
}

fn dir_size(path: &path::Path) -> u64 {
    walkdir::WalkDir::new(path)
        .follow_links(SYMLINK_FOLLOW)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|e| e.len())
        .sum()
}

pub fn pretty_size(size: u64) -> String {
    let size = size;
    const KIBIBYTE: u64 = 1024;
    const MEBIBYTE: u64 = 1_048_576;
    const GIBIBYTE: u64 = 1_073_741_824;
    const TEBIBYTE: u64 = 1_099_511_627_776;
    const PEBIBYTE: u64 = 1_125_899_906_842_624;
    const EXBIBYTE: u64 = 1_152_921_504_606_846_976;

    let (size, symbol) = match size {
        size if size < KIBIBYTE => (size as f64, "B"),
        size if size < MEBIBYTE => (size as f64 / KIBIBYTE as f64, "KiB"),
        size if size < GIBIBYTE => (size as f64 / MEBIBYTE as f64, "MiB"),
        size if size < TEBIBYTE => (size as f64 / GIBIBYTE as f64, "GiB"),
        size if size < PEBIBYTE => (size as f64 / TEBIBYTE as f64, "TiB"),
        size if size < EXBIBYTE => (size as f64 / PEBIBYTE as f64, "PiB"),
        _ => (size as f64 / EXBIBYTE as f64, "EiB"),
    };

    format!("{:.1}{}", size, symbol)
}
