use std::{
    borrow::Cow,
    collections::{HashSet, VecDeque},
    error::{self, Error},
    fs,
    path::{self, Path, PathBuf},
    time::SystemTime,
};

const FILE_CARGO_TOML: &str = "Cargo.toml";
const FILE_PACKAGE_JSON: &str = "package.json";
const FILE_ASSEMBLY_CSHARP: &str = "Assembly-CSharp.csproj";
const FILE_STACK_HASKELL: &str = "stack.yaml";
const FILE_CABAL_HASKELL: &str = "cabal.project";
const FILE_SBT_BUILD: &str = "build.sbt";
const FILE_MVN_BUILD: &str = "pom.xml";
const FILE_BUILD_GRADLE: &str = "build.gradle";
const FILE_BUILD_GRADLE_KTS: &str = "build.gradle.kts";
const FILE_CMAKE_BUILD: &str = "CMakeLists.txt";
const FILE_UNREAL_SUFFIX: &str = ".uproject";
const FILE_JUPYTER_SUFFIX: &str = ".ipynb";
const FILE_PYTHON_SUFFIX: &str = ".py";
const FILE_PIXI_PACKAGE: &str = "pixi.toml";
const FILE_COMPOSER_JSON: &str = "composer.json";
const FILE_PUBSPEC_YAML: &str = "pubspec.yaml";
const FILE_ELIXIR_MIX: &str = "mix.exs";
const FILE_SWIFT_PACKAGE: &str = "Package.swift";
const FILE_BUILD_ZIG: &str = "build.zig";
const FILE_GODOT_4_PROJECT: &str = "project.godot";
const FILE_CSPROJ_SUFFIX: &str = ".csproj";
const FILE_FSPROJ_SUFFIX: &str = ".fsproj";
const FILE_TERRAFORM_HCL: &str = ".terraform.lock.hcl";
const FILE_PROJECT_TURBOREPO: &str = "turbo.json";
const FILE_PODFILE: &str = "Podfile";

const PROJECT_CARGO_DIRS: [&str; 2] = ["target", ".xwin-cache"];
const PROJECT_NODE_DIRS: [&str; 2] = ["node_modules", ".angular"];
const PROJECT_REACT_NATIVE_DIRS: [&str; 8] = [
    "node_modules",
    "android/build",
    "android/.gradle",
    "ios/build",
    "ios/DerivedData",
    "ios/Pods",
    ".expo",
    ".metro",
];
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
const PROJECT_CABAL_DIRS: [&str; 1] = ["dist-newstyle"];
const PROJECT_SBT_DIRS: [&str; 2] = ["target", "project/target"];
const PROJECT_MVN_DIRS: [&str; 1] = ["target"];
const PROJECT_GRADLE_DIRS: [&str; 2] = ["build", ".gradle"];
const PROJECT_CMAKE_DIRS: [&str; 3] = ["build", "cmake-build-debug", "cmake-build-release"];
const PROJECT_UNREAL_DIRS: [&str; 5] = [
    "Binaries",
    "Build",
    "Saved",
    "DerivedDataCache",
    "Intermediate",
];
const PROJECT_JUPYTER_DIRS: [&str; 1] = [".ipynb_checkpoints"];
const PROJECT_PYTHON_DIRS: [&str; 7] = [
    ".mypy_cache",
    ".nox",
    ".pytest_cache",
    ".ruff_cache",
    ".tox",
    "__pycache__",
    "__pypackages__",
];
const PROJECT_PIXI_DIRS: [&str; 1] = [".pixi"];
const PROJECT_COMPOSER_DIRS: [&str; 1] = ["vendor"];
const PROJECT_PUB_DIRS: [&str; 4] = [
    "build",
    ".dart_tool",
    "linux/flutter/ephemeral",
    "windows/flutter/ephemeral",
];
const PROJECT_ELIXIR_DIRS: [&str; 4] = ["_build", ".elixir-tools", ".elixir_ls", ".lexical"];
const PROJECT_SWIFT_DIRS: [&str; 2] = [".build", ".swiftpm"];
const PROJECT_ZIG_DIRS: [&str; 3] = ["zig-cache", ".zig-cache", "zig-out"];
const PROJECT_GODOT_4_DIRS: [&str; 1] = [".godot"];
const PROJECT_DOTNET_DIRS: [&str; 2] = ["bin", "obj"];
const PROJECT_TURBOREPO_DIRS: [&str; 1] = [".turbo"];
const PROJECT_TERRAFORM_DIRS: [&str; 1] = [".terraform"];
const PROJECT_COCOAPODS_DIRS: [&str; 1] = ["Pods"];

const PROJECT_CARGO_NAME: &str = "Cargo";
const PROJECT_NODE_NAME: &str = "Node";
const PROJECT_NODE_REACT_NATIVE_NAME: &str = "Node (React Native)";
const PROJECT_UNITY_NAME: &str = "Unity";
const PROJECT_STACK_NAME: &str = "Stack";
const PROJECT_CABAL_NAME: &str = "Cabal";
const PROJECT_SBT_NAME: &str = "SBT";
const PROJECT_MVN_NAME: &str = "Maven";
const PROJECT_GRADLE_NAME: &str = "Gradle";
const PROJECT_CMAKE_NAME: &str = "CMake";
const PROJECT_UNREAL_NAME: &str = "Unreal";
const PROJECT_JUPYTER_NAME: &str = "Jupyter";
const PROJECT_PYTHON_NAME: &str = "Python";
const PROJECT_PIXI_NAME: &str = "Pixi";
const PROJECT_COMPOSER_NAME: &str = "Composer";
const PROJECT_PUB_NAME: &str = "Pub";
const PROJECT_ELIXIR_NAME: &str = "Elixir";
const PROJECT_SWIFT_NAME: &str = "Swift";
const PROJECT_ZIG_NAME: &str = "Zig";
const PROJECT_GODOT_4_NAME: &str = "Godot 4.x";
const PROJECT_DOTNET_NAME: &str = ".NET";
const PROJECT_TURBOREPO_NAME: &str = "Turborepo";
const PROJECT_TERRAFORM_NAME: &str = "Terraform";
const PROJECT_COCOAPODS_NAME: &str = "CocoaPods";

#[derive(Debug, Clone)]
pub enum ProjectType {
    Cargo,
    Node,
    Unity,
    Stack,
    Cabal,
    #[allow(clippy::upper_case_acronyms)]
    SBT,
    Maven,
    Gradle,
    CMake,
    Unreal,
    Jupyter,
    Python,
    Pixi,
    Composer,
    Pub,
    Elixir,
    Swift,
    Zig,
    Godot4,
    Dotnet,
    Turborepo,
    Terraform,
    Cocoapods,
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
            ProjectType::Node => {
                if is_react_native_project(&self.path) {
                    &PROJECT_REACT_NATIVE_DIRS
                } else {
                    &PROJECT_NODE_DIRS
                }
            }
            ProjectType::Unity => &PROJECT_UNITY_DIRS,
            ProjectType::Stack => &PROJECT_STACK_DIRS,
            ProjectType::Cabal => &PROJECT_CABAL_DIRS,
            ProjectType::SBT => &PROJECT_SBT_DIRS,
            ProjectType::Maven => &PROJECT_MVN_DIRS,
            ProjectType::Unreal => &PROJECT_UNREAL_DIRS,
            ProjectType::Jupyter => &PROJECT_JUPYTER_DIRS,
            ProjectType::Python => &PROJECT_PYTHON_DIRS,
            ProjectType::Pixi => &PROJECT_PIXI_DIRS,
            ProjectType::CMake => &PROJECT_CMAKE_DIRS,
            ProjectType::Composer => &PROJECT_COMPOSER_DIRS,
            ProjectType::Pub => &PROJECT_PUB_DIRS,
            ProjectType::Elixir => &PROJECT_ELIXIR_DIRS,
            ProjectType::Swift => &PROJECT_SWIFT_DIRS,
            ProjectType::Gradle => &PROJECT_GRADLE_DIRS,
            ProjectType::Zig => &PROJECT_ZIG_DIRS,
            ProjectType::Godot4 => &PROJECT_GODOT_4_DIRS,
            ProjectType::Dotnet => &PROJECT_DOTNET_DIRS,
            ProjectType::Turborepo => &PROJECT_TURBOREPO_DIRS,
            ProjectType::Terraform => &PROJECT_TERRAFORM_DIRS,
            ProjectType::Cocoapods => &PROJECT_COCOAPODS_DIRS,
        }
    }

    pub fn name(&self) -> Cow<'_, str> {
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
            ProjectType::Node => {
                if is_react_native_project(&self.path) {
                    PROJECT_NODE_REACT_NATIVE_NAME
                } else {
                    PROJECT_NODE_NAME
                }
            }
            ProjectType::Unity => PROJECT_UNITY_NAME,
            ProjectType::Stack => PROJECT_STACK_NAME,
            ProjectType::Cabal => PROJECT_CABAL_NAME,
            ProjectType::SBT => PROJECT_SBT_NAME,
            ProjectType::Maven => PROJECT_MVN_NAME,
            ProjectType::Unreal => PROJECT_UNREAL_NAME,
            ProjectType::Jupyter => PROJECT_JUPYTER_NAME,
            ProjectType::Python => PROJECT_PYTHON_NAME,
            ProjectType::Pixi => PROJECT_PIXI_NAME,
            ProjectType::CMake => PROJECT_CMAKE_NAME,
            ProjectType::Composer => PROJECT_COMPOSER_NAME,
            ProjectType::Pub => PROJECT_PUB_NAME,
            ProjectType::Elixir => PROJECT_ELIXIR_NAME,
            ProjectType::Swift => PROJECT_SWIFT_NAME,
            ProjectType::Gradle => PROJECT_GRADLE_NAME,
            ProjectType::Zig => PROJECT_ZIG_NAME,
            ProjectType::Godot4 => PROJECT_GODOT_4_NAME,
            ProjectType::Dotnet => PROJECT_DOTNET_NAME,
            ProjectType::Turborepo => PROJECT_TURBOREPO_NAME,
            ProjectType::Terraform => PROJECT_TERRAFORM_NAME,
            ProjectType::Cocoapods => PROJECT_COCOAPODS_NAME,
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
    const YEAR: u64 = DAY * 365;

    let (unit, fstring) = match secs {
        secs if secs < MINUTE => (secs as f64, "second"),
        secs if secs < HOUR * 2 => (secs as f64 / MINUTE as f64, "minute"),
        secs if secs < DAY * 2 => (secs as f64 / HOUR as f64, "hour"),
        secs if secs < WEEK * 2 => (secs as f64 / DAY as f64, "day"),
        secs if secs < MONTH * 2 => (secs as f64 / WEEK as f64, "week"),
        secs if secs < YEAR * 2 => (secs as f64 / MONTH as f64, "month"),
        secs => (secs as f64 / YEAR as f64, "year"),
    };

    let unit = unit.round();

    let plural = if unit == 1.0 { "" } else { "s" };

    format!("{unit:.0} {fstring}{plural} ago")
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name().to_string_lossy().starts_with('.')
}

/// Roots of the linked worktrees registered in `git_dir/worktrees`.
///
/// Locked worktrees, worktrees whose directory no longer exists and entries
/// that can't be read are skipped.
fn linked_worktrees(git_dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = git_dir.join("worktrees").read_dir() else {
        return Vec::new();
    };
    rd.filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|meta| !meta.join("locked").exists())
        .filter_map(|meta| {
            let raw = fs::read_to_string(meta.join("gitdir")).ok()?;
            // an absolute gitdir replaces `meta`, a relative one is resolved against it
            let dotgit = meta.join(raw.trim());
            if dotgit.file_name()? != ".git" {
                return None;
            }
            let root = dotgit.parent()?.to_path_buf();
            root.is_dir().then_some(root)
        })
        .collect()
}

/// Whether `dir`, which has a `.git` file, is a linked worktree that git has locked.
fn is_locked_worktree(dir: &Path) -> bool {
    let Ok(dotgit) = fs::read_to_string(dir.join(".git")) else {
        return false;
    };
    let Some(git_dir) = dotgit.trim().strip_prefix("gitdir:") else {
        return false;
    };
    dir.join(git_dir.trim()).join("locked").exists()
}

#[cfg(unix)]
fn device(path: &Path) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    fs::metadata(path).ok().map(|m| m.dev())
}

#[cfg(not(unix))]
fn device(_path: &Path) -> Option<u64> {
    None
}

fn walk(path: &Path, options: &ScanOptions) -> walkdir::IntoIter {
    walkdir::WalkDir::new(path)
        .follow_links(options.follow_symlinks)
        .same_file_system(options.same_file_system)
        .into_iter()
}

struct ProjectIter {
    it: walkdir::IntoIter,
    options: ScanOptions,
    /// The scan root as given by the caller, used to build the paths of worktree walks.
    display_root: PathBuf,
    /// Canonical scan root, worktrees outside of it are never walked.
    root: PathBuf,
    root_dev: Option<u64>,
    /// Whether the current walk started from a queued worktree root.
    queued_walk: bool,
    pending: VecDeque<PathBuf>,
    queued: HashSet<PathBuf>,
    emitted: HashSet<PathBuf>,
}

impl ProjectIter {
    fn new(path: &Path, options: &ScanOptions) -> Self {
        let display_root = path.to_path_buf();
        let root = if options.worktrees {
            fs::canonicalize(path).unwrap_or_else(|_| display_root.clone())
        } else {
            display_root.clone()
        };
        let root_dev = if options.worktrees && options.same_file_system {
            device(&root)
        } else {
            None
        };
        Self {
            it: walk(path, options),
            options: options.clone(),
            queued: HashSet::from([root.clone()]),
            display_root,
            root,
            root_dev,
            queued_walk: false,
            pending: VecDeque::new(),
            emitted: HashSet::new(),
        }
    }

    fn queue_worktrees(&mut self, git_dir: &Path) {
        for worktree in linked_worktrees(git_dir) {
            let Ok(canonical) = fs::canonicalize(&worktree) else {
                continue;
            };
            let Ok(relative) = canonical.strip_prefix(&self.root) else {
                continue;
            };
            if self.root_dev.is_some() && device(&canonical) != self.root_dev {
                continue;
            }
            let walk_root = self.display_root.join(relative);
            if self.queued.insert(canonical) {
                self.pending.push_back(walk_root);
            }
        }
    }

    /// Returns false if a project at `path` has already been returned by this scan.
    fn first_emit(&mut self, path: &Path) -> bool {
        if !self.options.worktrees {
            return true;
        }
        // Without following symlinks every walked path is built from real directory names
        // under `display_root`, so equal paths are equal directories.
        let key = if self.options.follow_symlinks {
            fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
        } else {
            path.to_path_buf()
        };
        self.emitted.insert(key)
    }
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
                None => {
                    let next_root = self.pending.pop_front()?;
                    self.it = walk(&next_root, &self.options);
                    self.queued_walk = true;
                    continue;
                }
                Some(Err(e)) => return Some(Err(Red::WalkdirError(e))),
                Some(Ok(entry)) => entry,
            };
            if !entry.file_type().is_dir() {
                continue;
            }
            if is_hidden(&entry) && !(self.queued_walk && entry.depth() == 0) {
                self.it.skip_current_dir();
                continue;
            }
            let rd = match entry.path().read_dir() {
                Err(e) => return Some(Err(Red::IOError(e))),
                Ok(rd) => rd,
            };
            // intentionally ignoring errors while iterating the ReadDir
            // can't return them because we'll lose the context of where we are
            let listing: Vec<_> = rd
                .filter_map(|rd| rd.ok())
                .filter_map(|de| Some((de.file_name(), de.file_type().ok()?)))
                .collect();
            if self.options.worktrees {
                match listing.iter().find(|(name, _)| name == ".git") {
                    Some((_, file_type)) if file_type.is_dir() => {
                        self.queue_worktrees(&entry.path().join(".git"));
                    }
                    Some((_, file_type))
                        if file_type.is_file() && is_locked_worktree(entry.path()) =>
                    {
                        self.it.skip_current_dir();
                        continue;
                    }
                    _ => {}
                }
            }
            for dir_entry in listing
                .iter()
                .filter(|(_, file_type)| file_type.is_file())
                .map(|(name, _)| name)
            {
                let file_name = match dir_entry.to_str() {
                    None => continue,
                    Some(file_name) => file_name,
                };
                let p_type = match file_name {
                    FILE_CARGO_TOML => Some(ProjectType::Cargo),
                    FILE_PACKAGE_JSON => Some(ProjectType::Node),
                    FILE_ASSEMBLY_CSHARP => Some(ProjectType::Unity),
                    FILE_STACK_HASKELL => Some(ProjectType::Stack),
                    FILE_CABAL_HASKELL => Some(ProjectType::Cabal),
                    FILE_SBT_BUILD => Some(ProjectType::SBT),
                    FILE_MVN_BUILD => Some(ProjectType::Maven),
                    FILE_CMAKE_BUILD => Some(ProjectType::CMake),
                    FILE_COMPOSER_JSON => Some(ProjectType::Composer),
                    FILE_PUBSPEC_YAML => Some(ProjectType::Pub),
                    FILE_PIXI_PACKAGE => Some(ProjectType::Pixi),
                    FILE_ELIXIR_MIX => Some(ProjectType::Elixir),
                    FILE_SWIFT_PACKAGE => Some(ProjectType::Swift),
                    FILE_BUILD_GRADLE => Some(ProjectType::Gradle),
                    FILE_BUILD_GRADLE_KTS => Some(ProjectType::Gradle),
                    FILE_BUILD_ZIG => Some(ProjectType::Zig),
                    FILE_GODOT_4_PROJECT => Some(ProjectType::Godot4),
                    FILE_PROJECT_TURBOREPO => Some(ProjectType::Turborepo),
                    FILE_TERRAFORM_HCL => Some(ProjectType::Terraform),
                    FILE_PODFILE => Some(ProjectType::Cocoapods),
                    file_name if file_name.ends_with(FILE_UNREAL_SUFFIX) => {
                        Some(ProjectType::Unreal)
                    }
                    file_name if file_name.ends_with(FILE_JUPYTER_SUFFIX) => {
                        Some(ProjectType::Jupyter)
                    }
                    file_name if file_name.ends_with(FILE_PYTHON_SUFFIX) => {
                        Some(ProjectType::Python)
                    }
                    file_name
                        if file_name.ends_with(FILE_CSPROJ_SUFFIX)
                            || file_name.ends_with(FILE_FSPROJ_SUFFIX) =>
                    {
                        if dir_contains_file(entry.path(), FILE_GODOT_4_PROJECT) {
                            Some(ProjectType::Godot4)
                        } else if dir_contains_file(entry.path(), FILE_ASSEMBLY_CSHARP) {
                            Some(ProjectType::Unity)
                        } else {
                            Some(ProjectType::Dotnet)
                        }
                    }
                    _ => None,
                };
                if let Some(project_type) = p_type {
                    self.it.skip_current_dir();
                    if !self.first_emit(entry.path()) {
                        break;
                    }
                    return Some(Ok(Project {
                        project_type,
                        path: entry.path().to_path_buf(),
                    }));
                }
            }
        }
    }
}

fn dir_contains_file(path: &Path, file: &str) -> bool {
    path.read_dir()
        .map(|rd| {
            rd.filter_map(|rd| rd.ok()).any(|de| {
                de.file_type().is_ok_and(|t| t.is_file()) && de.file_name().to_str() == Some(file)
            })
        })
        .unwrap_or(false)
}

fn dir_contains_subdir(path: &Path, subdir: &str) -> bool {
    path.read_dir()
        .map(|rd| {
            rd.filter_map(|rd| rd.ok()).any(|de| {
                de.file_type().is_ok_and(|t| t.is_dir()) && de.file_name().to_str() == Some(subdir)
            })
        })
        .unwrap_or(false)
}

fn is_react_native_project(path: &Path) -> bool {
    dir_contains_subdir(path, "ios") || dir_contains_subdir(path, "android")
}

#[derive(Clone, Debug)]
pub struct ScanOptions {
    pub follow_symlinks: bool,
    pub same_file_system: bool,
    /// Also scan the linked git worktrees registered by repositories found under the scan
    /// root, even when they live in hidden directories or inside another project.
    /// Locked worktrees are skipped, wherever they are found.
    pub worktrees: bool,
}

pub fn scan<P: AsRef<path::Path>>(
    path: &P,
    options: &ScanOptions,
) -> impl Iterator<Item = Result<Project, Red>> {
    ProjectIter::new(path.as_ref(), options)
}

// TODO does this need to exist as is??
pub fn dir_size<P: AsRef<path::Path>>(path: &P, options: &ScanOptions) -> u64 {
    walk(path.as_ref(), options)
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
                FILE_CABAL_HASKELL => Some(ProjectType::Cabal),
                FILE_SBT_BUILD => Some(ProjectType::SBT),
                FILE_MVN_BUILD => Some(ProjectType::Maven),
                FILE_CMAKE_BUILD => Some(ProjectType::CMake),
                FILE_COMPOSER_JSON => Some(ProjectType::Composer),
                FILE_PUBSPEC_YAML => Some(ProjectType::Pub),
                FILE_PIXI_PACKAGE => Some(ProjectType::Pixi),
                FILE_ELIXIR_MIX => Some(ProjectType::Elixir),
                FILE_SWIFT_PACKAGE => Some(ProjectType::Swift),
                FILE_BUILD_ZIG => Some(ProjectType::Zig),
                FILE_GODOT_4_PROJECT => Some(ProjectType::Godot4),
                FILE_TERRAFORM_HCL => Some(ProjectType::Terraform),
                FILE_PODFILE => Some(ProjectType::Cocoapods),
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn elapsed() {
        assert_eq!(print_elapsed(0), "0 seconds ago");
        assert_eq!(print_elapsed(1), "1 second ago");
        assert_eq!(print_elapsed(2), "2 seconds ago");
        assert_eq!(print_elapsed(59), "59 seconds ago");
        assert_eq!(print_elapsed(60), "1 minute ago");
        assert_eq!(print_elapsed(61), "1 minute ago");
        assert_eq!(print_elapsed(119), "2 minutes ago");
        assert_eq!(print_elapsed(120), "2 minutes ago");
        assert_eq!(print_elapsed(121), "2 minutes ago");
        assert_eq!(print_elapsed(3599), "60 minutes ago");
        assert_eq!(print_elapsed(3600), "60 minutes ago");
        assert_eq!(print_elapsed(3601), "60 minutes ago");
        assert_eq!(print_elapsed(7199), "120 minutes ago");
        assert_eq!(print_elapsed(7200), "2 hours ago");
        assert_eq!(print_elapsed(7201), "2 hours ago");
        assert_eq!(print_elapsed(86399), "24 hours ago");
        assert_eq!(print_elapsed(86400), "24 hours ago");
        assert_eq!(print_elapsed(86401), "24 hours ago");
        assert_eq!(print_elapsed(172799), "48 hours ago");
        assert_eq!(print_elapsed(172800), "2 days ago");
        assert_eq!(print_elapsed(172801), "2 days ago");
        assert_eq!(print_elapsed(604799), "7 days ago");
        assert_eq!(print_elapsed(604800), "7 days ago");
        assert_eq!(print_elapsed(604801), "7 days ago");
        assert_eq!(print_elapsed(1209599), "14 days ago");
        assert_eq!(print_elapsed(1209600), "2 weeks ago");
        assert_eq!(print_elapsed(1209601), "2 weeks ago");
        assert_eq!(print_elapsed(2419199), "4 weeks ago");
        assert_eq!(print_elapsed(2419200), "4 weeks ago");
        assert_eq!(print_elapsed(2419201), "4 weeks ago");
        assert_eq!(print_elapsed(2419200 * 2), "2 months ago");
        assert_eq!(print_elapsed(2419200 * 3), "3 months ago");
        assert_eq!(print_elapsed(2419200 * 12), "12 months ago");
        assert_eq!(print_elapsed(2419200 * 25), "25 months ago");
        assert_eq!(print_elapsed(2419200 * 48), "4 years ago");
    }

    /// A scan root whose name isn't hidden, unlike `tempfile::tempdir()`.
    fn tempdir() -> tempfile::TempDir {
        tempfile::Builder::new().prefix("kondo").tempdir().unwrap()
    }

    fn options(worktrees: bool) -> ScanOptions {
        ScanOptions {
            follow_symlinks: false,
            same_file_system: false,
            worktrees,
        }
    }

    fn scan_paths(root: &Path, options: &ScanOptions) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = scan(&root, options)
            .map(|result| match result {
                Ok(project) => project.path,
                Err(Red::IOError(e)) => panic!("unexpected io error: {e}"),
                Err(Red::WalkdirError(e)) => panic!("unexpected walkdir error: {e}"),
            })
            .collect();
        paths.sort();
        paths
    }

    fn write(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn cargo_project(dir: &Path) {
        write(&dir.join("Cargo.toml"), "[package]\n");
        write(&dir.join("target/debug/out"), "artifact");
    }

    fn repo(dir: &Path) {
        cargo_project(dir);
        fs::create_dir_all(dir.join(".git/objects")).unwrap();
    }

    /// Registers `worktree` in `repo` with `gitdir` contents, as `git worktree add` does.
    fn register(repo: &Path, id: &str, gitdir: &str) -> PathBuf {
        let meta = repo.join(".git/worktrees").join(id);
        write(&meta.join("gitdir"), &format!("{gitdir}\n"));
        meta
    }

    fn linked_worktree(repo: &Path, id: &str, worktree: &Path) -> PathBuf {
        cargo_project(worktree);
        let meta = repo.join(".git/worktrees").join(id);
        write(
            &worktree.join(".git"),
            &format!("gitdir: {}\n", meta.display()),
        );
        register(repo, id, &worktree.join(".git").to_string_lossy())
    }

    #[test]
    fn worktree_in_hidden_dir_inside_project() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        let w1 = repo_dir.join(".claude/worktrees/w1");
        repo(&repo_dir);
        linked_worktree(&repo_dir, "w1", &w1);

        assert_eq!(scan_paths(tmp.path(), &options(true)), [repo_dir, w1]);
    }

    #[test]
    fn worktree_with_relative_gitdir() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        let w1 = repo_dir.join(".claude/worktrees/w1");
        repo(&repo_dir);
        cargo_project(&w1);
        register(&repo_dir, "w1", "../../../.claude/worktrees/w1/.git");

        assert_eq!(scan_paths(tmp.path(), &options(true)), [repo_dir, w1]);
    }

    #[test]
    fn worktree_whose_dir_was_deleted() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        let w1 = repo_dir.join(".claude/worktrees/w1");
        repo(&repo_dir);
        linked_worktree(&repo_dir, "w1", &w1);
        fs::remove_dir_all(&w1).unwrap();

        assert_eq!(scan_paths(tmp.path(), &options(true)), [repo_dir]);
    }

    #[test]
    fn worktree_outside_scan_root() {
        let tmp = tempdir();
        let root = tmp.path().join("root");
        let repo_dir = root.join("repo");
        repo(&repo_dir);
        linked_worktree(&repo_dir, "w1", &tmp.path().join("outside/w1"));

        assert_eq!(scan_paths(&root, &options(true)), [repo_dir]);
    }

    #[test]
    fn locked_worktree() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        repo(&repo_dir);
        let meta = linked_worktree(&repo_dir, "w1", &repo_dir.join(".claude/worktrees/w1"));
        write(&meta.join("locked"), "claude agent agent-1234 (pid 42)");

        assert_eq!(scan_paths(tmp.path(), &options(true)), [repo_dir]);
    }

    #[test]
    fn locked_worktree_reached_by_walk() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        let feature = tmp.path().join("repo-feature");
        let nested = tmp.path().join("repo-nested");
        repo(&repo_dir);
        let meta = linked_worktree(&repo_dir, "repo-feature", &feature);
        write(&meta.join("locked"), "");
        let meta = register(&repo_dir, "repo-nested", "../../../../repo-nested/.git");
        write(&meta.join("locked"), "");
        cargo_project(&nested.join("crates/a"));
        write(
            &nested.join(".git"),
            "gitdir: ../repo/.git/worktrees/repo-nested\n",
        );

        assert_eq!(
            scan_paths(tmp.path(), &options(false)),
            [repo_dir.clone(), feature, nested.join("crates/a")]
        );
        assert_eq!(scan_paths(tmp.path(), &options(true)), [repo_dir]);
    }

    #[test]
    fn sibling_worktree_reached_by_walk() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        let feature = tmp.path().join("repo-feature");
        repo(&repo_dir);
        linked_worktree(&repo_dir, "repo-feature", &feature);

        assert_eq!(scan_paths(tmp.path(), &options(true)), [repo_dir, feature]);
    }

    #[test]
    fn worktrees_disabled() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        repo(&repo_dir);
        linked_worktree(&repo_dir, "w1", &repo_dir.join(".claude/worktrees/w1"));

        assert_eq!(scan_paths(tmp.path(), &options(false)), [repo_dir]);
    }

    #[test]
    fn hidden_worktree_root_is_scanned_but_not_hidden_dirs_below_it() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        let worktree = repo_dir.join(".tmp-w1");
        repo(&repo_dir);
        linked_worktree(&repo_dir, "tmp-w1", &worktree);
        cargo_project(&worktree.join("crates/a"));
        fs::remove_file(worktree.join("Cargo.toml")).unwrap();
        cargo_project(&worktree.join(".hidden/b"));

        assert_eq!(
            scan_paths(tmp.path(), &options(true)),
            [repo_dir, worktree.join("crates/a")]
        );
    }

    #[test]
    fn hidden_worktree_root_with_marker() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        let worktree = repo_dir.join(".tmp-w1");
        repo(&repo_dir);
        linked_worktree(&repo_dir, "tmp-w1", &worktree);

        assert_eq!(scan_paths(tmp.path(), &options(true)), [repo_dir, worktree]);
    }

    #[test]
    fn two_registrations_of_the_same_worktree() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        let w1 = repo_dir.join(".claude/worktrees/w1");
        repo(&repo_dir);
        linked_worktree(&repo_dir, "w1", &w1);
        register(&repo_dir, "w1-again", "../../../.claude/worktrees/w1/.git");

        assert_eq!(scan_paths(tmp.path(), &options(true)), [repo_dir, w1]);
    }

    #[test]
    fn registration_pointing_back_at_the_repo() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        repo(&repo_dir);
        register(&repo_dir, "self", &repo_dir.join(".git").to_string_lossy());
        register(
            &repo_dir,
            "root",
            &tmp.path().join(".git").to_string_lossy(),
        );

        assert_eq!(scan_paths(tmp.path(), &options(true)), [repo_dir]);
    }

    #[test]
    fn worktrees_of_repo_without_marker() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        let w1 = repo_dir.join(".worktrees/w1");
        fs::create_dir_all(repo_dir.join(".git")).unwrap();
        linked_worktree(&repo_dir, "w1", &w1);

        assert_eq!(scan_paths(tmp.path(), &options(true)), [w1]);
    }

    #[test]
    fn worktree_in_dir_with_dot_git_file_is_not_read() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        cargo_project(&repo_dir);
        write(&repo_dir.join(".git"), "gitdir: ../elsewhere/.git\n");

        assert_eq!(scan_paths(tmp.path(), &options(true)), [repo_dir]);
    }

    #[cfg(unix)]
    #[test]
    fn worktree_also_reached_through_followed_symlink() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        let w1 = repo_dir.join(".claude/worktrees/w1");
        repo(&repo_dir);
        linked_worktree(&repo_dir, "w1", &w1);
        std::os::unix::fs::symlink(&w1, tmp.path().join("link")).unwrap();

        let options = ScanOptions {
            follow_symlinks: true,
            ..options(true)
        };
        let paths = scan_paths(tmp.path(), &options);
        assert_eq!(paths.len(), 2, "{paths:?}");
        assert!(paths.contains(&repo_dir));
    }

    #[test]
    fn linked_worktrees_reads_registry() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        let git_dir = repo_dir.join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        let w1 = tmp.path().join("w1");
        fs::create_dir_all(&w1).unwrap();
        fs::create_dir_all(tmp.path().join("not-dotgit")).unwrap();

        register(
            &repo_dir,
            "w1",
            &format!("  {}  ", w1.join(".git").display()),
        );
        register(&repo_dir, "bad-name", "../../../../not-dotgit/gitdir");
        register(&repo_dir, "empty", "");
        register(
            &repo_dir,
            "gone",
            &tmp.path().join("gone/.git").to_string_lossy(),
        );
        fs::create_dir_all(git_dir.join("worktrees/no-gitdir")).unwrap();
        write(&git_dir.join("worktrees/stray-file"), "");
        write(&git_dir.join("worktrees/not-utf8/gitdir"), "");
        fs::write(git_dir.join("worktrees/not-utf8/gitdir"), [0xff, 0xfe]).unwrap();

        assert_eq!(linked_worktrees(&git_dir), [w1]);
        assert!(linked_worktrees(&tmp.path().join("missing")).is_empty());
    }

    fn set_modified(path: &Path, time: SystemTime) {
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(time)
            .unwrap();
    }

    fn project_at(root: &Path, path: &Path) -> Project {
        scan(&root, &options(true))
            .filter_map(Result::ok)
            .find(|p| p.path == path)
            .unwrap()
    }

    fn in_a_month() -> SystemTime {
        SystemTime::now() + Duration::from_secs(60 * 60 * 24 * 30)
    }

    #[test]
    fn worktree_last_modified_ignores_gitignored_artifacts() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        let w1 = repo_dir.join(".claude/worktrees/w1");
        repo(&repo_dir);
        linked_worktree(&repo_dir, "w1", &w1);
        write(&w1.join(".gitignore"), "target/\n");
        let future = in_a_month();
        set_modified(&w1.join("target/debug/out"), future);

        let project = project_at(tmp.path(), &w1);
        assert!(project.last_modified(&options(true)).unwrap() < future);
    }

    #[test]
    fn worktree_dot_git_file_makes_it_a_repo_root_for_last_modified() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        let feature = tmp.path().join("repo-feature");
        repo(&repo_dir);
        linked_worktree(&repo_dir, "repo-feature", &feature);
        write(&feature.join(".gitignore"), "target/\n");
        let future = in_a_month();
        set_modified(&feature.join("target/debug/out"), future);

        let project = project_at(tmp.path(), &feature);
        assert!(project.last_modified(&options(true)).unwrap() < future);

        // outside of a git repo the gitignore is not applied
        fs::remove_file(feature.join(".git")).unwrap();
        assert_eq!(project.last_modified(&options(true)).unwrap(), future);
    }

    #[test]
    fn worktree_last_modified_is_not_hidden_by_parent_repo_gitignore() {
        let tmp = tempdir();
        let repo_dir = tmp.path().join("repo");
        let w1 = repo_dir.join(".claude/worktrees/w1");
        repo(&repo_dir);
        write(&repo_dir.join(".gitignore"), ".claude/\nsrc/\n");
        linked_worktree(&repo_dir, "w1", &w1);
        write(&w1.join("src/main.rs"), "fn main() {}\n");
        let future = in_a_month();
        set_modified(&w1.join("src/main.rs"), future);

        let project = project_at(tmp.path(), &w1);
        assert_eq!(project.last_modified(&options(true)).unwrap(), future);
    }
}
