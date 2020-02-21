use std::{env, io, path, process};
use structopt::StructOpt;
use walkdir;

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

fn check_file_exists(
    path: &path::Path,
    file_name: &str,
    project_type: ProjectType,
) -> Option<Project> {
    let has_cargo_toml = path.read_dir().unwrap().any(|r| match r {
        Ok(de) => de.file_name() == file_name,
        Err(_) => false,
    });
    if has_cargo_toml {
        return Some(Project {
            project_type: project_type,
            path: path.to_path_buf(),
        });
    }
    None
}

fn cargo_project(path: &path::Path) -> Option<Project> {
    check_file_exists(path, FILE_CARGO_TOML, ProjectType::Cargo)
}

fn node_project(path: &path::Path) -> Option<Project> {
    check_file_exists(path, FILE_PACKAGE_JSON, ProjectType::Node)
}

fn sbt_project(path: &path::Path) -> Option<Project> {
    check_file_exists(path, FILE_SBT_BUILD, ProjectType::SBT)
}

fn unity_project(path: &path::Path) -> Option<Project> {
    check_file_exists(path, FILE_ASSEMBLY_CSHARP, ProjectType::Unity)
}

fn stack_project(path: &path::Path) -> Option<Project> {
    check_file_exists(path, FILE_STACK_HASKELL, ProjectType::Stack)
}

fn mvn_project(path: &path::Path) -> Option<Project> {
    check_file_exists(path, FILE_MVN_BUILD, ProjectType::Maven)
}

const PROJECT_TYPES: [fn(path: &path::Path) -> Option<Project>; 6] = [
    cargo_project,
    node_project,
    unity_project,
    stack_project,
    sbt_project,
    mvn_project,
];

enum ProjectType {
    Cargo,
    Node,
    Unity,
    Stack,
    SBT,
    Maven,
}

struct Project {
    project_type: ProjectType,
    path: path::PathBuf,
}

impl Project {
    fn artifact_dirs(&self) -> impl Iterator<Item = &&str> {
        match self.project_type {
            ProjectType::Cargo => PROJECT_CARGO_DIRS.iter(),
            ProjectType::Node => PROJECT_NODE_DIRS.iter(),
            ProjectType::Unity => PROJECT_UNITY_DIRS.iter(),
            ProjectType::Stack => PROJECT_STACK_DIRS.iter(),
            ProjectType::SBT => PROJECT_SBT_DIRS.iter(),
            ProjectType::Maven => PROJECT_MVN_DIRS.iter(),
        }
    }

    fn name(&self) -> String {
        self.path.to_str().unwrap().to_string()
    }

    fn size(&self) -> u64 {
        self.artifact_dirs()
            .map(|p| dir_size(&self.path.join(p)))
            .sum()
    }

    fn type_name(&self) -> &str {
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

fn scan<P: AsRef<path::Path>>(path: &P) -> impl Iterator<Item = Project> {
    walkdir::WalkDir::new(path)
        .follow_links(SYMLINK_FOLLOW)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .filter_map(|dir| {
            let dir = dir.path();
            PROJECT_TYPES.iter().find_map(|p| p(dir))
        })
}

fn dir_size(path: &path::Path) -> u64 {
    walkdir::WalkDir::new(path)
        .follow_links(SYMLINK_FOLLOW)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e: walkdir::DirEntry| e.metadata())
        .filter_map(|md| md.ok())
        .map(|e| e.len())
        .sum()
}

fn pretty_size(size: u64) -> String {
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

#[derive(StructOpt, Debug)]
#[structopt(name = "kondo")]
struct Opt {
    /// Output artifact directories only
    #[structopt(short, long)]
    artifact_dirs: bool,

    /// Run command for artifact dirs
    #[structopt(short, long)]
    command: Option<String>,

    /// The directories to examine
    #[structopt(name = "DIRS", parse(from_os_str))]
    dirs: Vec<std::path::PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use io::Write;
    let opt = Opt::from_args();
    let dirs: Vec<path::PathBuf> = {
        let cd = env::current_dir()?;
        if opt.dirs.is_empty() {
            vec![cd]
        } else {
            opt.dirs
                .into_iter()
                .map(|d| if d.is_absolute() { d } else { cd.join(d) })
                .collect()
        }
    };

    let project_dirs: Vec<Project> = dirs.iter().flat_map(scan).collect();

    let stdout = io::stdout();
    let mut write_handle = stdout.lock();

    if let Some(command) = opt.command {
        for dir in project_dirs.iter() {
            let dir_base = &dir.path;
            for p in dir.artifact_dirs() {
                process::Command::new(&command)
                    .arg(dir_base.join(p))
                    .spawn()?;
            }
        }
        return Ok(());
    };

    if opt.artifact_dirs {
        for dir in project_dirs.iter() {
            let dir_base = &dir.path;
            for p in dir.artifact_dirs() {
                writeln!(&mut write_handle, "{}", dir_base.join(p).to_string_lossy())?;
            }
        }
        return Ok(());
    }

    let mut total = 0;

    let mut project_sizes: Vec<(u64, String, &str)> = project_dirs
        .iter()
        .map(|p| {
            let size = p.size();
            total += size;
            (size, p.name(), p.type_name())
        })
        .filter(|(size, _, _)| *size > 0)
        .collect();

    project_sizes.sort_unstable_by_key(|p| p.0);

    for (size, name, type_name) in project_sizes.iter() {
        writeln!(
            &mut write_handle,
            "{:>10} {} {}",
            pretty_size(*size),
            type_name,
            name
        )?;
    }

    writeln!(&mut write_handle, "{} possible savings", pretty_size(total))?;

    Ok(())
}
