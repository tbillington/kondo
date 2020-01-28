use std::{env, io, path};
use walkdir;

const SYMLINK_FOLLOW: bool = true;

const FILE_CARGO_TOML: &str = "Cargo.toml";
const FILE_PACKAGE_JSON: &str = "package.json";
const FILE_ASSEMBLY_CSHARP: &str = "Assembly-CSharp.csproj";
const FILE_SBT_BUILD: &str = "build.sbt";

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
const PROJECT_SBT_DIRS: [&str; 2] = [
    "target",
    "project/target",
];

#[derive(Clone, Debug)]
enum ProjectType {
    Cargo,
    Node,
    Unity,
    Sbt,
}

#[derive(Clone, Debug)]
struct ProjectDir {
    r#type: ProjectType,
    path: path::PathBuf,
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn scan<P: AsRef<path::Path>>(path: &P) -> Vec<ProjectDir> {
    walkdir::WalkDir::new(path)
        .follow_links(SYMLINK_FOLLOW)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            if entry.file_type().is_file() {
                Some(ProjectDir {
                    r#type: match entry.file_name().to_str() {
                        Some(FILE_CARGO_TOML) => ProjectType::Cargo,
                        Some(FILE_PACKAGE_JSON) => ProjectType::Node,
                        Some(FILE_ASSEMBLY_CSHARP) => ProjectType::Unity,
                        Some(FILE_SBT_BUILD) => ProjectType::Sbt,
                        _ => return None,
                    },
                    path: entry
                        .path()
                        .parent()
                        .expect("it's a file, so it should definitely have a parent...")
                        .to_path_buf(),
                })
            } else {
                None
            }
        })
        .collect()
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
    let size = size as f64;
    const KIBIBYTE: f64 = 1024.0;
    const MEBIBYTE: f64 = 1_048_576.0;
    const GIBIBYTE: f64 = 1_073_741_824.0;
    const TEBIBYTE: f64 = 1_099_511_627_776.0;
    const PEBIBYTE: f64 = 1_125_899_906_842_624.0;
    const EXBIBYTE: f64 = 1_152_921_504_606_846_976.0;

    let (size, symbol) = if size < KIBIBYTE {
        (size, "B")
    } else if size < MEBIBYTE {
        (size / KIBIBYTE, "KiB")
    } else if size < GIBIBYTE {
        (size / MEBIBYTE, "MiB")
    } else if size < TEBIBYTE {
        (size / GIBIBYTE, "GiB")
    } else if size < PEBIBYTE {
        (size / TEBIBYTE, "TiB")
    } else if size < EXBIBYTE {
        (size / PEBIBYTE, "PiB")
    } else {
        (size / EXBIBYTE, "EiB")
    };

    format!("{:.1}{}", size, symbol)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use io::Write;
    let dir = {
        let mut args: Vec<String> = env::args().collect();
        if args.len() == 2 {
            path::PathBuf::from(args.pop().unwrap())
        } else {
            env::current_dir()?
        }
    };

    let stdout = io::stdout();
    let mut write_handle = stdout.lock();

    writeln!(&mut write_handle, "Scanning {:?}", dir)?;

    let mut project_dirs = scan(&dir);

    {
        // Remove child directories if they have a parent in the list
        let mut i = 0;
        'outer: while i < project_dirs.len() {
            let mut j = i + 1;

            while j < project_dirs.len() {
                let (p1, p2) = (&project_dirs[i].path, &project_dirs[j].path);

                if p1.starts_with(p2) {
                    project_dirs.remove(i);
                    continue 'outer;
                } else if p2.starts_with(p1) {
                    project_dirs.remove(j);
                    continue;
                }

                j += 1;
            }

            i += 1;
        }
    }

    writeln!(&mut write_handle, "{} projects found", project_dirs.len())?;

    writeln!(&mut write_handle, "Calculating savings per project")?;

    let mut total = 0;

    let mut project_sizes: Vec<(u64, String)> = project_dirs
        .iter()
        .map(|ProjectDir { r#type, path }| {
            let (name, dirs) = match r#type {
                ProjectType::Cargo => ("Cargo", PROJECT_CARGO_DIRS.iter()),
                ProjectType::Node => ("Node", PROJECT_NODE_DIRS.iter()),
                ProjectType::Unity => ("Unity", PROJECT_UNITY_DIRS.iter()),
                ProjectType::Sbt => ("SBT", PROJECT_SBT_DIRS.iter()),
            };

            let size = dirs.map(|p| dir_size(&path.join(p))).sum();
            total += size;

            (
                size,
                format!(
                    "{} ({}) {}",
                    path.strip_prefix(&dir)
                        .unwrap()
                        .file_name()
                        .map(|n| n.to_str().unwrap())
                        .unwrap_or("."),
                    name,
                    path.display()
                ),
            )
        })
        .filter(|(size, _)| *size > 1)
        .collect();

    project_sizes.sort_unstable_by_key(|p| p.0);

    for (size, project) in project_sizes.iter() {
        writeln!(&mut write_handle, "{:>10} {}", pretty_size(*size), project)?;
    }

    writeln!(&mut write_handle, "{} possible savings", pretty_size(total))?;

    Ok(())
}
