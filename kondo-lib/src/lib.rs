use std::{
    borrow::Cow,
    error::{self, Error},
    fs, path,
    time::SystemTime,
};

const FILE_CARGO_TOML: &str = "Cargo.toml";
const FILE_PACKAGE_JSON: &str = "package.json";
const FILE_ASSEMBLY_CSHARP: &str = "Assembly-CSharp.csproj";
const FILE_STACK_HASKELL: &str = "stack.yaml";
const FILE_SBT_BUILD: &str = "build.sbt";
const FILE_MVN_BUILD: &str = "pom.xml";
const FILE_CMAKE_BUILD: &str = "CMakeLists.txt";
const FILE_UNREAL_SUFFIX: &str = ".uproject";
const FILE_JUPYTER_SUFFIX: &str = ".ipynb";
const FILE_PYTHON_SUFFIX: &str = ".py";
const FILE_COMPOSER_JSON: &str = "composer.json";
const FILE_PUBSPEC_YAML: &str = "pubspec.yaml";
const FILE_ELIXIR_MIX: &str = "mix.exs";
const FILE_SWIFT_PACKAGE: &str = "Package.swift";

const PROJECT_CARGO_DIRS: [&str; 1] = ["target"];
const PROJECT_NODE_DIRS: [&str; 2] = ["node_modules", ".angular"];
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
const PROJECT_CMAKE_DIRS: [&str; 1] = ["build"];
const PROJECT_UNREAL_DIRS: [&str; 5] = [
    "Binaries",
    "Build",
    "Saved",
    "DerivedDataCache",
    "Intermediate",
];
const PROJECT_JUPYTER_DIRS: [&str; 1] = [".ipynb_checkpoints"];
const PROJECT_PYTHON_DIRS: [&str; 3] = ["__pycache__", "__pypackages__", ".venv"];
const PROJECT_COMPOSER_DIRS: [&str; 1] = ["vendor"];
const PROJECT_PUB_DIRS: [&str; 4] = [
    "build",
    ".dart_tool",
    "linux/flutter/ephemeral",
    "windows/flutter/ephemeral",
];
const PROJECT_ELIXIR_DIRS: [&str; 1] = ["_build"];
const PROJECT_SWIFT_DIRS: [&str; 2] = [".build", ".swiftpm"];

const PROJECT_CARGO_NAME: &str = "Cargo";
const PROJECT_NODE_NAME: &str = "Node";
const PROJECT_UNITY_NAME: &str = "Unity";
const PROJECT_STACK_NAME: &str = "Stack";
const PROJECT_SBT_NAME: &str = "SBT";
const PROJECT_MVN_NAME: &str = "Maven";
const PROJECT_CMAKE_NAME: &str = "CMake";
const PROJECT_UNREAL_NAME: &str = "Unreal";
const PROJECT_JUPYTER_NAME: &str = "Jupyter";
const PROJECT_PYTHON_NAME: &str = "Python";
const PROJECT_COMPOSER_NAME: &str = "Composer";
const PROJECT_PUB_NAME: &str = "Pub";
const PROJECT_ELIXIR_NAME: &str = "Elixir";
const PROJECT_SWIFT_NAME: &str = "Swift";

#[derive(Debug, Clone)]
pub enum ProjectType {
    Cargo,
    Node,
    Unity,
    Stack,
    #[allow(clippy::upper_case_acronyms)]
    SBT,
    Maven,
    CMake,
    Unreal,
    Jupyter,
    Python,
    Composer,
    Pub,
    Elixir,
    Swift,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub project_type: ProjectType,
    pub path: path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct ProjectSize {
    pub artifact_size: u64,
    pub non_artifact_size: u64,
    pub dirs: Vec<(String, u64, bool)>,
}

impl Project {
    pub fn artifact_dirs(&self) -> &[&str] {
        match self.project_type {
            ProjectType::Cargo => &PROJECT_CARGO_DIRS,
            ProjectType::Node => &PROJECT_NODE_DIRS,
            ProjectType::Unity => &PROJECT_UNITY_DIRS,
            ProjectType::Stack => &PROJECT_STACK_DIRS,
            ProjectType::SBT => &PROJECT_SBT_DIRS,
            ProjectType::Maven => &PROJECT_MVN_DIRS,
            ProjectType::Unreal => &PROJECT_UNREAL_DIRS,
            ProjectType::Jupyter => &PROJECT_JUPYTER_DIRS,
            ProjectType::Python => &PROJECT_PYTHON_DIRS,
            ProjectType::CMake => &PROJECT_CMAKE_DIRS,
            ProjectType::Composer => &PROJECT_COMPOSER_DIRS,
            ProjectType::Pub => &PROJECT_PUB_DIRS,
            ProjectType::Elixir => &PROJECT_ELIXIR_DIRS,
            ProjectType::Swift => &PROJECT_SWIFT_DIRS,
        }
    }

    pub fn name(&self) -> Cow<str> {
        self.path.to_string_lossy()
    }

    pub fn size(&self, options: &ScanOptions) -> u64 {
        self.artifact_dirs()
            .iter()
            .copied()
            .map(|p| dir_size(&self.path.join(p), options))
            .sum()
    }

    pub fn last_modified(&self, options: &ScanOptions) -> Result<SystemTime, std::io::Error> {
        let top_level_modified = fs::metadata(&self.path)?.modified()?;
        let most_recent_modified = ignore::WalkBuilder::new(&self.path)
            .follow_links(options.follow_symlinks)
            .same_file_system(options.same_file_system)
            .build()
            .fold(top_level_modified, |acc, e| {
                if let Ok(e) = e {
                    if let Ok(e) = e.metadata() {
                        if let Ok(modified) = e.modified() {
                            if modified > acc {
                                return modified;
                            }
                        }
                    }
                }
                acc
            });
        Ok(most_recent_modified)
    }

    pub fn size_dirs(&self, options: &ScanOptions) -> ProjectSize {
        let mut artifact_size = 0;
        let mut non_artifact_size = 0;
        let mut dirs = Vec::new();

        let project_root = match fs::read_dir(&self.path) {
            Err(_) => {
                return ProjectSize {
                    artifact_size,
                    non_artifact_size,
                    dirs,
                }
            }
            Ok(rd) => rd,
        };

        for entry in project_root.filter_map(|rd| rd.ok()) {
            let file_type = match entry.file_type() {
                Err(_) => continue,
                Ok(file_type) => file_type,
            };

            if file_type.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    non_artifact_size += metadata.len();
                }
                continue;
            }

            if file_type.is_dir() {
                let file_name = match entry.file_name().into_string() {
                    Err(_) => continue,
                    Ok(file_name) => file_name,
                };
                let size = dir_size(&entry.path(), options);
                let artifact_dir = self.artifact_dirs().contains(&file_name.as_str());
                if artifact_dir {
                    artifact_size += size;
                } else {
                    non_artifact_size += size;
                }
                dirs.push((file_name, size, artifact_dir));
            }
        }

        ProjectSize {
            artifact_size,
            non_artifact_size,
            dirs,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self.project_type {
            ProjectType::Cargo => PROJECT_CARGO_NAME,
            ProjectType::Node => PROJECT_NODE_NAME,
            ProjectType::Unity => PROJECT_UNITY_NAME,
            ProjectType::Stack => PROJECT_STACK_NAME,
            ProjectType::SBT => PROJECT_SBT_NAME,
            ProjectType::Maven => PROJECT_MVN_NAME,
            ProjectType::Unreal => PROJECT_UNREAL_NAME,
            ProjectType::Jupyter => PROJECT_JUPYTER_NAME,
            ProjectType::Python => PROJECT_PYTHON_NAME,
            ProjectType::CMake => PROJECT_CMAKE_NAME,
            ProjectType::Composer => PROJECT_COMPOSER_NAME,
            ProjectType::Pub => PROJECT_PUB_NAME,
            ProjectType::Elixir => PROJECT_ELIXIR_NAME,
            ProjectType::Swift => PROJECT_SWIFT_NAME,
        }
    }

    /// Deletes the project's artifact directories and their contents
    pub fn clean(&self) {
        for artifact_dir in self
            .artifact_dirs()
            .iter()
            .copied()
            .map(|ad| self.path.join(ad))
            .filter(|ad| ad.exists())
        {
            if let Err(e) = fs::remove_dir_all(&artifact_dir) {
                eprintln!("error removing directory {:?}: {:?}", artifact_dir, e);
            }
        }
    }
}

pub fn print_elapsed(secs: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;
    const WEEK: u64 = DAY * 7;
    const MONTH: u64 = WEEK * 4;
    const YEAR: u64 = MONTH * 12;

    let (unit, fstring) = match secs {
        secs if secs < MINUTE => (secs as f64, "second"),
        secs if secs < HOUR * 2 => (secs as f64 / MINUTE as f64, "minute"),
        secs if secs < DAY * 2 => (secs as f64 / HOUR as f64, "hour"),
        secs if secs < WEEK * 2 => (secs as f64 / DAY as f64, "day"),
        secs if secs < MONTH * 2 => (secs as f64 / WEEK as f64, "week"),
        secs if secs < YEAR * 2 => (secs as f64 / MONTH as f64, "month"),
        secs => (secs as f64 / MONTH as f64, "year"),
    };

    let unit = unit.round();

    let plural = if unit == 1.0 { "" } else { "s" };

    format!("{unit:.0} {fstring}{plural} ago")
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name().to_string_lossy().starts_with('.')
}

struct ProjectIter {
    it: walkdir::IntoIter,
}

pub enum Red {
    IOError(::std::io::Error),
    WalkdirError(walkdir::Error),
}

impl Iterator for ProjectIter {
    type Item = Result<Project, Red>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry: walkdir::DirEntry = match self.it.next() {
                None => return None,
                Some(Err(e)) => return Some(Err(Red::WalkdirError(e))),
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
                Err(e) => return Some(Err(Red::IOError(e))),
                Ok(rd) => rd,
            };
            // intentionally ignoring errors while iterating the ReadDir
            // can't return them because we'll lose the context of where we are
            for dir_entry in rd.filter_map(|rd| rd.ok()).map(|de| de.file_name()) {
                let file_name = match dir_entry.to_str() {
                    None => continue,
                    Some(file_name) => file_name,
                };
                let p_type = match file_name {
                    FILE_CARGO_TOML => Some(ProjectType::Cargo),
                    FILE_PACKAGE_JSON => Some(ProjectType::Node),
                    FILE_ASSEMBLY_CSHARP => Some(ProjectType::Unity),
                    FILE_STACK_HASKELL => Some(ProjectType::Stack),
                    FILE_SBT_BUILD => Some(ProjectType::SBT),
                    FILE_MVN_BUILD => Some(ProjectType::Maven),
                    FILE_CMAKE_BUILD => Some(ProjectType::CMake),
                    FILE_COMPOSER_JSON => Some(ProjectType::Composer),
                    FILE_PUBSPEC_YAML => Some(ProjectType::Pub),
                    FILE_ELIXIR_MIX => Some(ProjectType::Elixir),
                    FILE_SWIFT_PACKAGE => Some(ProjectType::Swift),
                    file_name if file_name.ends_with(FILE_UNREAL_SUFFIX) => {
                        Some(ProjectType::Unreal)
                    }
                    file_name if file_name.ends_with(FILE_JUPYTER_SUFFIX) => {
                        Some(ProjectType::Jupyter)
                    }
                    file_name if file_name.ends_with(FILE_PYTHON_SUFFIX) => {
                        Some(ProjectType::Python)
                    }
                    _ => None,
                };
                if let Some(project_type) = p_type {
                    self.it.skip_current_dir();
                    return Some(Ok(Project {
                        project_type,
                        path: entry.path().to_path_buf(),
                    }));
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScanOptions {
    pub follow_symlinks: bool,
    pub same_file_system: bool,
}

fn build_walkdir_iter<P: AsRef<path::Path>>(path: &P, options: &ScanOptions) -> ProjectIter {
    ProjectIter {
        it: walkdir::WalkDir::new(path)
            .follow_links(options.follow_symlinks)
            .same_file_system(options.same_file_system)
            .into_iter(),
    }
}

pub fn scan<P: AsRef<path::Path>>(
    path: &P,
    options: &ScanOptions,
) -> impl Iterator<Item = Result<Project, Red>> {
    build_walkdir_iter(path, options)
}

// TODO does this need to exist as is??
pub fn dir_size<P: AsRef<path::Path>>(path: &P, options: &ScanOptions) -> u64 {
    build_walkdir_iter(path, options)
        .it
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|e| e.len())
        .sum()
}

pub fn pretty_size(size: u64) -> String {
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

pub fn clean(project_path: &str) -> Result<(), Box<dyn error::Error>> {
    let project = fs::read_dir(project_path)?
        .filter_map(|rd| rd.ok())
        .find_map(|dir_entry| {
            let file_name = dir_entry.file_name().into_string().ok()?;
            let p_type = match file_name.as_str() {
                FILE_CARGO_TOML => Some(ProjectType::Cargo),
                FILE_PACKAGE_JSON => Some(ProjectType::Node),
                FILE_ASSEMBLY_CSHARP => Some(ProjectType::Unity),
                FILE_STACK_HASKELL => Some(ProjectType::Stack),
                FILE_SBT_BUILD => Some(ProjectType::SBT),
                FILE_MVN_BUILD => Some(ProjectType::Maven),
                FILE_CMAKE_BUILD => Some(ProjectType::CMake),
                FILE_COMPOSER_JSON => Some(ProjectType::Composer),
                FILE_PUBSPEC_YAML => Some(ProjectType::Pub),
                FILE_ELIXIR_MIX => Some(ProjectType::Elixir),
                FILE_SWIFT_PACKAGE => Some(ProjectType::Swift),
                _ => None,
            };
            if let Some(project_type) = p_type {
                return Some(Project {
                    project_type,
                    path: project_path.into(),
                });
            }
            None
        });

    if let Some(project) = project {
        for artifact_dir in project
            .artifact_dirs()
            .iter()
            .copied()
            .map(|ad| path::PathBuf::from(project_path).join(ad))
            .filter(|ad| ad.exists())
        {
            if let Err(e) = fs::remove_dir_all(&artifact_dir) {
                eprintln!("error removing directory {:?}: {:?}", artifact_dir, e);
            }
        }
    }

    Ok(())
}
pub fn path_canonicalise(
    base: &path::Path,
    tail: path::PathBuf,
) -> Result<path::PathBuf, Box<dyn Error>> {
    if tail.is_absolute() {
        Ok(tail)
    } else {
        Ok(base.join(tail).canonicalize()?)
    }
}
