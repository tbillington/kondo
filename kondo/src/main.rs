use structopt::StructOpt;

use std::{
    env::current_dir,
    error::Error,
    io::{stdin, stdout, BufRead, Write},
    path::PathBuf,
};

use kondo_lib::{dir_size, path_canonicalise, pretty_size, print_elapsed, scan, ScanOptions};

#[derive(StructOpt, Debug)]
#[structopt(name = "kondo")]
/// Kondo recursively cleans project directories.
///
/// Supported project types: Cargo, Node, Unity, SBT, Haskell Stack, Maven, Unreal Engine, Jupyter Notebook, and Python projects.
struct Opt {
    /// The directories to examine. Current directory will be used if DIRS is omitted.
    #[structopt(name = "DIRS", parse(from_os_str))]
    dirs: Vec<PathBuf>,

    /// Quiet mode. Won't output to the terminal. -qq prevents all output.
    #[structopt(short, long, parse(from_occurrences))]
    quiet: u8,

    /// Clean all found projects without confirmation.
    #[structopt(short, long)]
    all: bool,

    /// Follow symbolic links
    #[structopt(short = "L", long)]
    follow_symlinks: bool,

    /// Restrict directory traversal to the root filesystem
    #[structopt(short, long)]
    same_filesystem: bool,
}

fn prepare_directories(dirs: Vec<PathBuf>) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let cd = current_dir()?;
    if dirs.is_empty() {
        return Ok(vec![cd]);
    }

    let dirs = dirs
        .into_iter()
        .filter_map(|path| {
            let exists = path.try_exists().unwrap_or(false);
            if !exists {
                eprintln!("error: directory {} does not exist", path.to_string_lossy());
                return None;
            }

            if let Ok(metadata) = path.metadata() {
                if metadata.is_file() {
                    eprintln!(
                        "error: file supplied but directory expected: {}",
                        path.to_string_lossy()
                    );
                    return None;
                }
            }

            path_canonicalise(&cd, path).ok()
        })
        .collect();

    Ok(dirs)
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    if opt.quiet > 0 && !opt.all {
        eprintln!("Quiet mode can only be used with --all.");
        std::process::exit(1);
    }

    let stdout = stdout();
    let mut write_handle = stdout.lock();
    let mut write_buffer = String::with_capacity(2048);

    let stdin = stdin();
    let mut read_handle = stdin.lock();

    let dirs = prepare_directories(opt.dirs)?;
    let mut projects_cleaned = 0;
    let mut bytes_deleted = 0;

    let mut clean_all = opt.all;

    let scan_options: ScanOptions = ScanOptions {
        follow_symlinks: opt.follow_symlinks,
        same_file_system: opt.same_filesystem,
    };

    'project_loop: for project in dirs
        .iter()
        .flat_map(|dir| scan(dir, &scan_options))
        .filter_map(|p| p.ok())
    {
        write_buffer.clear();

        let project_artifact_bytes = project
            .artifact_dirs()
            .iter()
            .copied()
            .filter_map(
                |dir| match dir_size(&project.path.join(dir), &scan_options) {
                    0 => None,
                    size => Some((dir, size)),
                },
            )
            .map(|(dir, size)| {
                write_buffer.push_str("\n  └─ ");
                write_buffer.push_str(dir);
                write_buffer.push_str(" (");
                write_buffer.push_str(&pretty_size(size));
                write_buffer.push(')');
                size
            })
            .sum::<u64>();

        if project_artifact_bytes == 0 {
            continue;
        }

        if opt.quiet == 0 {
            let mut last_modified_str = String::new();

            if let Ok(last_modified) = project.last_modified(&scan_options) {
                if let Ok(elapsed) = last_modified.elapsed() {
                    let elapsed = print_elapsed(elapsed.as_secs());
                    last_modified_str = format!("({elapsed})");
                }
            }

            writeln!(
                &mut write_handle,
                "{} {} project {last_modified_str}{}",
                &project.name(),
                project.type_name(),
                write_buffer
            )?;
        }

        let clean_project = if clean_all {
            true
        } else {
            loop {
                write!(
                    &mut write_handle,
                    "  delete above artifact directories? ([y]es, [n]o, [a]ll, [q]uit): "
                )?;
                write_handle.flush()?;
                let mut choice = String::new();
                read_handle.read_line(&mut choice)?;
                match choice.trim_end() {
                    "y" => break true,
                    "n" => break false,
                    "a" => {
                        clean_all = true;
                        break true;
                    }
                    "q" => {
                        writeln!(&mut write_handle)?;
                        break 'project_loop;
                    }
                    _ => writeln!(
                        &mut write_handle,
                        "  invalid choice, please choose between y, n, a, or q."
                    )?,
                }
            }
        };

        if clean_project {
            project.clean();
            if opt.quiet == 0 {
                writeln!(
                    &mut write_handle,
                    "  deleted {}",
                    &pretty_size(project_artifact_bytes)
                )?;
            }
            bytes_deleted += project_artifact_bytes;
            projects_cleaned += 1;
        }

        if opt.quiet == 0 {
            writeln!(&mut write_handle)?;
        }
    }

    if opt.quiet < 2 {
        writeln!(
            &mut write_handle,
            "Projects cleaned: {}, Bytes deleted: {}",
            projects_cleaned,
            pretty_size(bytes_deleted)
        )?;
    }

    Ok(())
}
