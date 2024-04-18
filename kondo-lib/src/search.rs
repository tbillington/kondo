use std::path::PathBuf;

use crossbeam_channel::Sender;
use crossbeam_deque::{Injector, Stealer, Worker};
use crossbeam_utils::{atomic::AtomicCell, sync::Parker};

use crate::{project::Project, ProjectEnum};

pub(crate) type Res = (PathBuf, ProjectEnum);

#[allow(clippy::too_many_arguments)]
pub(crate) fn search_thread(
    i: &usize,
    local: Worker<PathBuf>,
    global: &Injector<PathBuf>,
    stealers: &[Stealer<PathBuf>],
    parker: Parker,
    active: &AtomicCell<bool>,
    finished: &AtomicCell<bool>,
    result_sender: &Sender<Res>,
    project_filter: &[ProjectEnum],
) {
    let mut scan_count: u32 = 0;
    let mut found_projects = Vec::new();
    // let mut non_artifact_dirs = Vec::new();
    loop {
        while let Some(current_task) = find_task(&local, global, stealers) {
            // println!("Searching {}", current_task.display());

            if !current_task.is_dir() {
                continue;
            }

            let path = current_task;

            // println!("Scanning {}", path.display());

            scan_count += 1;

            for p in project_filter.iter() {
                if p.is_project(&path) {
                    found_projects.push((path.clone(), *p));
                    // println!("Found project: {} {}", p.name(), path.display());
                }
            }

            if found_projects.is_empty() {
                if let Ok(dir) = path.read_dir() {
                    for entry in dir.flatten() {
                        if entry.path().is_dir() {
                            local.push(entry.path());
                        }
                    }
                }
                continue;
            }

            let Ok(dir) = path.read_dir() else {
                continue;
            };

            dir.flatten().for_each(|entry| {
                if !entry.path().is_dir() {
                    return;
                }

                if found_projects
                    .iter()
                    .all(|(_, p)| !p.is_artifact(&entry.path()))
                {
                    local.push(entry.path());
                    // non_artifact_dirs.push(entry.path());
                }
            });

            found_projects.drain(..).for_each(|(path, p)| {
                // println!("Found project: {} {}", p.kind_name(), path.display());
                if result_sender.send((path, p)).is_err() {
                    // Should this be a separate Atomic Var ? User has dropped the channel
                    finished.store(true);
                }
            });

            continue;

            // let Ok(read_dir) = current_task.read_dir() else {
            //     continue;
            // };

            // read_dir.flatten().for_each(|entry| {
            //     let path = entry.path();
            //     if !path.is_dir() {
            //         return;
            //     }

            //     println!("Scanning {}", path.display());

            //     scan_count += 1;
            //     for p in PROJECTS.iter() {
            //         if p.is_project(&path) {
            //             found_projects.push((path.clone(), *p));
            //             // println!("Found project: {} {}", p.name(), path.display());
            //         }
            //     }

            //     if found_projects.is_empty() {
            //         return;
            //     }

            //     // non_artifact_dirs.extend(path.read_dir().unwrap().filter_map(|e| {
            //     //     e.ok().map(|e| {
            //     //         e.file_type()
            //     //             .ok()
            //     //             .map(|ef| {
            //     //                 ef.is_dir()
            //     //                     && found_projects
            //     //                         .iter()
            //     //                         .all(|(_, p)| !p.is_artifact(&e.path()))
            //     //             })
            //     //             .map(|_| e.path())
            //     //     })?
            //     // }));

            //     non_artifact_dirs.drain(..).for_each(|path| {
            //         local.push(path);
            //     });

            //     found_projects.drain(..).for_each(|(path, p)| {
            //         result_sender.send((path, p)).unwrap();
            //     });
            // });

            // for entry in WalkDir::new(&current_task).max_depth(1) {
            //     if let Ok(entry) = entry {
            //         if !entry.path().is_dir() {
            //             continue;
            //         }

            //         for p in PROJECTS.iter() {
            //             if p.is_project(entry.path()) {
            //                 found_projects.push((entry.path().to_path_buf(), *p));
            //                 // println!("Found project: {} {}", p.name(), entry.path().display());
            //             }
            //         }

            //         if found_projects.is_empty() {
            //             continue;
            //         }

            //         non_artifact_dirs.extend(entry.path().read_dir().unwrap().filter_map(|e| {
            //             e.ok().map(|e| {
            //                 e.file_type()
            //                     .ok()
            //                     .map(|ef| {
            //                         ef.is_dir()
            //                             && found_projects
            //                                 .iter()
            //                                 .all(|(_, p)| !p.is_artifact(&e.path()))
            //                     })
            //                     .map(|_| e.path())
            //             })?
            //         }));

            //         non_artifact_dirs.drain(..).for_each(|path| {
            //             local.push(path);
            //         });

            //         found_projects.drain(..).for_each(|(path, p)| {
            //             result_sender.send((path, p)).unwrap();
            //         });

            //         // break;

            //         // let directories_without_artifacts = found_projects
            //         //     .iter()
            //         //     .flat_map(|p| p.artifact_dirs())
            //         //     .copied()
            //         //     .collect::<Vec<_>>();

            //         // println!("  {}", entry.path().display());
            //         // if entry.path().is_dir() {
            //         //     local.push(entry.path().to_owned());
            //         // }
            //     } else {
            //         //
            //     }
            // }
        }

        // println!("Parking");
        active.store(false);
        parker.park();
        // println!("Unparking");
        if finished.load() {
            // println!("Exiting");
            break;
        }
        active.store(true);
    }
    // println!("Thread {} scanned {} dirs", i, scan_count);
}

fn find_task(
    local: &Worker<PathBuf>,
    global: &Injector<PathBuf>,
    stealers: &[Stealer<PathBuf>],
) -> Option<PathBuf> {
    // Pop a task from the local queue, if not empty.
    local.pop().or_else(|| {
        // Otherwise, we need to look for a task elsewhere.
        std::iter::repeat_with(|| {
            // Try stealing a batch of tasks from the global queue.
            global
                .steal_batch_and_pop(local)
                // Or try stealing a task from one of the other threads.
                .or_else(|| stealers.iter().map(|s| s.steal()).collect())
        })
        // Loop while no task was stolen and any steal operation needs to be retried.
        .find(|s| !s.is_retry())
        // Extract the stolen task, if there is one.
        .and_then(|s| s.success())
    })
}
