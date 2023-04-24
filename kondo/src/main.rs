use std::{
    env::current_dir,
    error::Error,
    fmt,
    io::{stdin, stdout, Write},
    num::ParseIntError,
    path::PathBuf,
    sync::mpsc::{Receiver, Sender, SyncSender},
};

use clap::Parser;

use kondo_lib::{
    dir_size, path_canonicalise, pretty_size, print_elapsed, scan, Project, ScanOptions,
};

// Below needs updating every time a new project type is added!
#[derive(Parser, Debug)]
#[command(name = "kondo")]
/// Kondo recursively cleans project directories.
///
/// Supported project types: Cargo, Node, Unity, SBT, Haskell Stack, Maven, Unreal Engine, Jupyter Notebook, Python, Jupyter Notebooks,CMake, Composer, Pub, Elixir, Swift, and Gradle projects.
struct Opt {
    /// The directories to examine. Current directory will be used if DIRS is omitted.
    #[arg(name = "DIRS")]
    dirs: Vec<PathBuf>,

    /// Quiet mode. Won't output to the terminal. -qq prevents all output.
    #[arg(short, long, action = clap::ArgAction::Count, value_parser = clap::value_parser!(u8).range(0..3))]
    quiet: u8,

    /// Clean all found projects without confirmation.
    #[arg(short, long)]
    all: bool,

    /// Follow symbolic links
    #[arg(short = 'L', long)]
    follow_symlinks: bool,

    /// Restrict directory traversal to the root filesystem
    #[arg(short, long)]
    same_filesystem: bool,

    /// Only directories with a file last modified n units of time ago will be looked at. Ex: 20d. Units are m: minutes, h: hours, d: days, w: weeks, M: months and y: years.
    #[arg(short, long, value_parser = parse_age_filter, default_value = "0d")]
    older: u64,
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

#[derive(Debug)]
pub enum ParseAgeFilterError {
    ParseIntError(ParseIntError),
    InvalidUnit,
}

impl fmt::Display for ParseAgeFilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseAgeFilterError::ParseIntError(e) => e.fmt(f),
            ParseAgeFilterError::InvalidUnit => {
                "invalid age unit, must be one of m, h, d, w, M, y".fmt(f)
            }
        }
    }
}

impl From<ParseIntError> for ParseAgeFilterError {
    fn from(e: ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}

impl Error for ParseAgeFilterError {}

pub fn parse_age_filter(age_filter: &str) -> Result<u64, ParseAgeFilterError> {
    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;
    const WEEK: u64 = DAY * 7;
    const MONTH: u64 = WEEK * 4;
    const YEAR: u64 = MONTH * 12;

    let (digit_end, unit) = age_filter
        .char_indices()
        .last()
        .ok_or(ParseAgeFilterError::InvalidUnit)?;

    let multiplier = match unit {
        'm' => MINUTE,
        'h' => HOUR,
        'd' => DAY,
        'w' => WEEK,
        'M' => MONTH,
        'y' => YEAR,
        _ => return Err(ParseAgeFilterError::InvalidUnit),
    };

    let count = age_filter[..digit_end].parse::<u64>()?;
    let seconds = count * multiplier;
    Ok(seconds)
}

type DiscoverData = (Project, Vec<(String, u64)>, u64, String);
type DeleteData = (Project, u64);

fn discover(
    dirs: Vec<PathBuf>,
    scan_options: &ScanOptions,
    project_min_age: u64,
    result_sender: SyncSender<DiscoverData>,
) {
    for project in dirs
        .iter()
        .flat_map(|dir| scan(dir, scan_options))
        .filter_map(|p| p.ok())
    {
        let artifact_dir_sizes: Vec<_> = project
            .artifact_dirs()
            .iter()
            .copied()
            .filter_map(
                |dir| match dir_size(&project.path.join(dir), scan_options) {
                    0 => None,
                    size => Some((dir.to_owned(), size)),
                },
            )
            .collect();
        let project_artifact_bytes = artifact_dir_sizes.iter().map(|(_, bytes)| bytes).sum();

        if project_artifact_bytes == 0 {
            continue;
        }

        let mut last_modified_str = String::new();
        let mut last_modified_int: u64 = 0;

        if let Ok(last_modified) = project.last_modified(scan_options) {
            if let Ok(elapsed) = last_modified.elapsed() {
                last_modified_int = elapsed.as_secs();
                let elapsed = print_elapsed(last_modified_int);
                last_modified_str = format!("({elapsed})");
            }
        }

        if last_modified_int < project_min_age {
            continue;
        }

        if result_sender
            .send((
                project,
                artifact_dir_sizes,
                project_artifact_bytes,
                last_modified_str,
            ))
            .is_err()
        {
            // interactive prompt has finished, silently finish here
            break;
        }
    }
}

fn process_deletes(project_recv: Receiver<DeleteData>) -> Vec<(Project, u64)> {
    project_recv
        .into_iter()
        .map(|(project, artifact_bytes)| {
            project.clean();
            (project, artifact_bytes)
        })
        .collect()
}

fn interactive_prompt(
    projects_recv: Receiver<DiscoverData>,
    deletes_send: Sender<DeleteData>,
    quiet: u8,
    mut clean_all: bool,
) {
    'project_loop: for (project, artifact_dirs, artifact_bytes, last_modified) in projects_recv {
        if quiet == 0 {
            println!(
                "{} {} project {last_modified}",
                &project.name(),
                project.type_name(),
            );
            for (dir, size) in artifact_dirs {
                println!("  └─ {dir} ({})", pretty_size(size));
            }
        }

        let clean_project = if clean_all {
            true
        } else {
            loop {
                print!("  delete above artifact directories? ([y]es, [n]o, [a]ll, [q]uit): ");
                stdout().flush().unwrap();
                let mut choice = String::new();

                stdin().read_line(&mut choice).unwrap();
                match choice.trim_end() {
                    "y" => break true,
                    "n" => break false,
                    "a" => {
                        clean_all = true;
                        break true;
                    }
                    "q" => {
                        println!();
                        break 'project_loop;
                    }
                    _ => println!("  invalid choice, please choose between y, n, a, or q."),
                }
            }
        };

        if clean_project {
            // TODO: Return an error that indicates a partial failure, not a show stopper
            if let Err(e) = deletes_send.send((project, artifact_bytes)) {
                eprintln!(
                    "no further projects will be scanned, error sending to delete thread {e}"
                );
                break;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::parse();

    if opt.quiet > 0 && !opt.all {
        eprintln!("Quiet mode can only be used with --all.");
        std::process::exit(1);
    }

    let dirs = prepare_directories(opt.dirs)?;

    let scan_options: ScanOptions = ScanOptions {
        follow_symlinks: opt.follow_symlinks,
        same_file_system: opt.same_filesystem,
    };

    let (proj_discover_send, proj_discover_recv) = std::sync::mpsc::sync_channel::<DiscoverData>(5);
    let (proj_delete_send, proj_delete_recv) = std::sync::mpsc::channel::<(Project, u64)>();

    let project_min_age = opt.older;
    std::thread::spawn(move || {
        discover(dirs, &scan_options, project_min_age, proj_discover_send);
    });

    let delete_handle = std::thread::spawn(move || process_deletes(proj_delete_recv));

    interactive_prompt(proj_discover_recv, proj_delete_send, opt.quiet, opt.all);

    let delete_results = match delete_handle.join() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error in delete thread, {e:?}");
            std::process::exit(1);
        }
    };

    if opt.quiet < 2 {
        let projects_cleaned = delete_results.len();
        let bytes_deleted = delete_results.iter().map(|(_, bytes)| bytes).sum();
        println!(
            "Projects cleaned: {}, Bytes deleted: {}",
            projects_cleaned,
            pretty_size(bytes_deleted)
        );
    }

    Ok(())
}
