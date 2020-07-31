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
struct Opt {
    /// The directories to examine. Current directory will be used if DIRS is omitted.
    #[structopt(name = "DIRS", parse(from_os_str))]
    dirs: Vec<PathBuf>,
}

fn prepare_directories(dirs: Vec<PathBuf>) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let cd = current_dir()?;
    Ok(if dirs.is_empty() {
        vec![cd]
    } else {
        dirs.into_iter()
            .map(|d| path_canonicalise(&cd, d))
            .collect()
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    let stdout = stdout();
    let mut write_handle = stdout.lock();
    let mut write_buffer = String::with_capacity(2048);

    let stdin = stdin();
    let mut read_handle = stdin.lock();

    let dirs = prepare_directories(opt.dirs)?;
    let mut bytes_deleted = 0;

    let mut clean_all = false;

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
                write_buffer.push_str(")");
                size
            })
            .sum::<u64>();

        if project_artifact_bytes == 0 {
            continue;
        }

        writeln!(
            &mut write_handle,
            "{} {} project{}",
            &project.name(),
            project.type_name(),
            write_buffer
        )?;

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
            writeln!(
                &mut write_handle,
                "  deleted {}",
                &pretty_size(project_artifact_bytes)
            )?;
            bytes_deleted += project_artifact_bytes;
        }

        writeln!(&mut write_handle)?;
    }

    writeln!(
        &mut write_handle,
        "Total bytes deleted: {}",
        pretty_size(bytes_deleted)
    )?;

    Ok(())
}
