use std::path::PathBuf;

use kondo_lib::{crossbeam::Receiver, Project as _};

use crate::{pretty_size2, print_elapsed, TableEntry};

pub(crate) fn discover(dirs: Vec<PathBuf>) -> Receiver<TableEntry> {
    let rx = kondo_lib::run_local(dirs.into_iter(), None);
    let (ttx, rrx) = kondo_lib::crossbeam::unbounded();
    std::thread::spawn(move || {
        let mut get_id = {
            let mut next_id = 0;
            move || {
                let id = next_id;
                next_id += 1;
                id
            }
        };

        while let Ok((path, proj)) = rx.recv() {
            let name = proj
                .name(&path)
                .unwrap_or_else(|| {
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned()
                })
                .into_boxed_str();

            let focus = proj
                .project_focus(&path)
                .map(|focus| focus.into_boxed_str());

            let artifact_bytes = proj.artifact_size(&path);

            // if artifact_bytes == 0 {
            //     continue;
            // }

            let artifact_bytes_fmt = pretty_size2(artifact_bytes);

            let mut last_modified_secs = None;
            if let Ok(lm) = proj.last_modified(&path) {
                if let Ok(elapsed) = lm.elapsed() {
                    let secs = elapsed.as_secs();
                    last_modified_secs = Some((secs, print_elapsed(secs)));
                }
            }

            let path_str = path.to_string_lossy().into_owned().into_boxed_str();

            let path_chars = path_str.chars().count() as u16;

            let entry = TableEntry {
                id: get_id(),
                proj,
                name,
                focus,
                path,
                path_str,
                path_chars,
                artifact_bytes,
                artifact_bytes_fmt,
                last_modified_secs,
                staged: false,
                cleaned: false,
            };

            if ttx.send(entry).is_err() {
                break;
            }
        }
    });
    rrx
}
