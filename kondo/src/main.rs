use structopt::StructOpt;

use std::{collections, env, io, path, process};

use kondo_lib::{dir_size, path_canonicalise, pretty_size, scan, Project};

#[derive(StructOpt, Debug)]
#[structopt(name = "kondo")]
struct Opt {
    /// Output artifact directories only
    #[structopt(short, long)]
    artifact_dirs: bool,

    /// Limit to existing directories only
    #[structopt(short, long)]
    existing_dirs: bool,

    /// Run command for artifact dirs
    #[structopt(short, long)]
    command: Option<String>,

    /// Run subcommand
    #[structopt(subcommand)]
    subcommand: Option<Command>,

    /// The directories to examine
    #[structopt(name = "DIRS", parse(from_os_str))]
    dirs: Vec<std::path::PathBuf>,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Clean projects in specified paths
    Clean {
        /// Show projects that will be cleaned without actually cleaning them
        #[structopt(long)]
        dry_run: bool,

        /// The directories to examine
        #[structopt(name = "DIRS", parse(from_os_str))]
        dirs: Vec<std::path::PathBuf>,
    },
    /// Stats
    Stats {
        /// The directories to examine
        #[structopt(name = "DIRS", parse(from_os_str))]
        dirs: Vec<std::path::PathBuf>,
    },
}

fn prepare_directories(
    dirs: Vec<path::PathBuf>,
) -> Result<Vec<path::PathBuf>, Box<dyn std::error::Error>> {
    let cd = env::current_dir()?;
    Ok(if dirs.is_empty() {
        vec![cd]
    } else {
        dirs.into_iter()
            .map(|d| path_canonicalise(&cd, d))
            .collect()
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use io::Write;
    let opt = Opt::from_args();

    let stdout = io::stdout();
    let mut write_handle = stdout.lock();

    match opt.subcommand {
        Some(Command::Clean { dry_run, dirs }) => {
            let dirs = prepare_directories(dirs)?;
            let project_dirs = dirs.iter().flat_map(scan);
            let mut total = 0;
            let mut artifact_dirs = Vec::with_capacity(10); // pre-allocated vec to reduce allocations
            for project in project_dirs {
                writeln!(&mut write_handle, "{}", project.name())?;
                artifact_dirs.extend(
                    project
                        .artifact_dirs()
                        .map(|d| (d.to_string(), project.path.join(d))),
                );
                for (i, (name, path)) in artifact_dirs.iter().enumerate() {
                    write!(
                        &mut write_handle,
                        "  {}─ {}",
                        if i == dirs.len() - 1 { "└" } else { "├" },
                        name,
                    )?;
                    let size = dir_size(path);
                    total += size;
                    write!(&mut write_handle, " ({})", pretty_size(size))?;
                    if !dry_run {
                        project.clean();
                        write!(&mut write_handle, " ✔")?;
                    }
                    writeln!(&mut write_handle)?;
                }
                artifact_dirs.clear();
            }
            writeln!(&mut write_handle, "Disk saving: {}", pretty_size(total))?;
            Ok(())
        }
        Some(Command::Stats { dirs }) => {
            let dirs = prepare_directories(dirs)?;
            let mut project_type_counts: collections::HashMap<&str, (u32, u64, u64)> =
                collections::HashMap::with_capacity(10);
            let mut total_artifact_size: u64 = 0;
            let mut total_non_artifact_size: u64 = 0;
            let projects = dirs
                .iter()
                .flat_map(scan)
                .map(|p: Project| {
                    let project_sizes = p.size_dirs();
                    let (count, artifact_size, non_artifact_size) = project_type_counts
                        .entry(p.type_name())
                        .or_insert((0, 0, 0));
                    *count += 1;
                    *artifact_size += project_sizes.artifact_size;
                    *non_artifact_size += project_sizes.non_artifact_size;
                    total_artifact_size += project_sizes.artifact_size;
                    total_non_artifact_size += project_sizes.non_artifact_size;
                    (p, project_sizes)
                })
                .collect::<Vec<_>>();

            writeln!(
                &mut write_handle,
                "{} project(s)\n  {} / {} artifacts, {:.1}%\n  {} / {} essential, {:.1}%",
                projects.len(),
                pretty_size(total_artifact_size),
                pretty_size(total_artifact_size + total_non_artifact_size),
                total_artifact_size as f64 / (total_artifact_size + total_non_artifact_size) as f64
                    * 100.0,
                pretty_size(total_non_artifact_size),
                pretty_size(total_artifact_size + total_non_artifact_size),
                total_non_artifact_size as f64
                    / (total_artifact_size + total_non_artifact_size) as f64
                    * 100.0
            )?;

            for (project_type, (count, artifact_size, non_artifact_size)) in
                project_type_counts.iter().map(|e| (*e.0, *e.1))
            {
                writeln!(
                    &mut write_handle,
                    "{} {} project(s) ({:.0}%)\n  {} / {} artifacts, {:.1}%\n  {} / {} essential, {:.1}%",
                    count,
                    project_type,
                    count as f64 / projects.len() as f64 * 100.0,
                    pretty_size(artifact_size),
                    pretty_size(artifact_size + non_artifact_size),
                    artifact_size as f64 / (artifact_size + non_artifact_size) as f64 * 100.0,
                    pretty_size(non_artifact_size),
                    pretty_size(artifact_size + non_artifact_size),
                    non_artifact_size as f64 / (artifact_size + non_artifact_size) as f64 * 100.0
                )?;
            }
            Ok(())
        }
        None => {
            let dirs = prepare_directories(opt.dirs)?;
            let project_dirs: Vec<Project> = dirs.iter().flat_map(scan).collect();

            if let Some(command) = opt.command {
                for dir in project_dirs.iter() {
                    let dir_base = &dir.path;
                    for p in dir.artifact_dirs() {
                        let full_path = dir_base.join(p);
                        if !opt.existing_dirs || full_path.metadata().is_ok() {
                            process::Command::new(&command).arg(full_path).spawn()?;
                        }
                    }
                }
                return Ok(());
            };

            if opt.artifact_dirs {
                for dir in project_dirs.iter() {
                    let dir_base = &dir.path;
                    for p in dir.artifact_dirs() {
                        let full_path = dir_base.join(p);
                        if !opt.existing_dirs || full_path.metadata().is_ok() {
                            writeln!(&mut write_handle, "{}", full_path.to_string_lossy())?;
                        }
                    }
                }
                return Ok(());
            }

            let mut total = 0;

            let mut project_sizes: Vec<(u64, String, &str)> = project_dirs
                .iter()
                .flat_map(|p| match p.size() {
                    0 => None,
                    size => {
                        total += size;
                        Some((size, p.name(), p.type_name()))
                    }
                })
                .collect();

            project_sizes.sort_unstable_by_key(|p| p.0);

            for (size, name, type_name) in project_sizes.iter() {
                writeln!(
                    &mut write_handle,
                    "{:>10} {:<6} {}",
                    pretty_size(*size),
                    type_name,
                    name
                )?;
            }

            writeln!(&mut write_handle, "{} possible savings", pretty_size(total))?;

            Ok(())
        }
    }
}
