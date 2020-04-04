use structopt::StructOpt;

use std::{env, io, path, process};

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
    /// List projects in specified paths
    List,
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
                .map(|d| {
                    if d.is_absolute() {
                        d
                    } else {
                        cd.join(d).canonicalize().expect("Unable to canonicalize!")
                    }
                })
                .collect()
        }
    };

    let stdout = io::stdout();
    let mut write_handle = stdout.lock();

    let project_dirs: Vec<Project> = dirs.iter().flat_map(scan).collect();

    match opt.subcommand {
        Some(Command::Clean { dry_run, dirs }) => {
            let cd = env::current_dir()?;
            let dirs = if dirs.is_empty() {
                vec![cd]
            } else {
                dirs.into_iter()
                    .map(|d| path_canonicalise(&cd, d))
                    .collect()
            };
            let projects = dirs.iter().flat_map(scan).map(|p: Project| {
                (
                    p.name(),
                    p.artifact_dirs()
                        .map(|d| (d.to_string(), p.path.join(d)))
                        .collect::<Vec<_>>(),
                )
            });
            let mut total = 0;
            if dry_run {
                for (name, dirs) in projects {
                    writeln!(&mut write_handle, "{}", name)?;
                    for (i, (name, path)) in dirs.iter().enumerate() {
                        write!(
                            &mut write_handle,
                            "  {}─ {} ",
                            if i == dirs.len() - 1 { "└" } else { "├" },
                            name,
                        )?;
                        let size = dir_size(path);
                        total += size;
                        writeln!(&mut write_handle, "({})", pretty_size(size))?;
                    }
                }
                writeln!(&mut write_handle, "Disk saving: {}", pretty_size(total))?;
            } else {
            }
            return Ok(());
        }
        Some(Command::List) => {
            println!("List");
            return Ok(());
        }
        None => {}
    }

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
