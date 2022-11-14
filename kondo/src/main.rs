use structopt::StructOpt;

use std::{
    env::current_dir,
    error::Error,
    fmt,
    io::{stdin, stdout, BufRead, Write},
    num::ParseIntError,
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

    /// Only directories with a file last modified n units of time ago will be looked at. Ex: 20d. Units are m: minutes, h: hours, d: days, w: weeks, M: months and y: years.
    #[structopt(short, long, parse(try_from_str = parse_age_filter), default_value = "0d")]
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

        let mut last_modified_str = String::new();
        let mut last_modified_int: u64 = 0;

        if let Ok(last_modified) = project.last_modified(&scan_options) {
            if let Ok(elapsed) = last_modified.elapsed() {
                last_modified_int = elapsed.as_secs();
                let elapsed = print_elapsed(last_modified_int);
                last_modified_str = format!("({elapsed})");
            }
        }

        if last_modified_int < opt.older {
            continue;
        }

        if opt.quiet == 0 {
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
