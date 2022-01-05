use structopt::StructOpt;

use std::{
    env::current_dir,
    error::Error,
    io::{stdin, stdout, BufRead, Write},
    path::PathBuf,
};

use kondo_lib::{dir_size, path_canonicalise, pretty_size, scan};

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
}

fn prepare_directories(dirs: Vec<PathBuf>) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let cd = current_dir()?;
    if dirs.is_empty() {
        return Ok(vec![cd]);
    }

    dirs.into_iter()
        .map(|d| path_canonicalise(&cd, d))
        .collect()
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

    'project_loop: for project in dirs.iter().flat_map(scan) {
        write_buffer.clear();

        let project_artifact_bytes = project
            .artifact_dirs()
            .iter()
            .copied()
            .filter_map(|dir| match dir_size(&project.path.join(dir)) {
                0 => None,
                size => Some((dir, size)),
            })
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
            writeln!(
                &mut write_handle,
                "{} {} project{}",
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
