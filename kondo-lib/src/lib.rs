mod project;
mod search;

#[cfg(test)]
mod test;

use std::{path::PathBuf, thread::available_parallelism, time::Duration};

use crossbeam::unbounded;
use crossbeam_deque::{Injector, Worker};
use crossbeam_utils::{atomic::AtomicCell, sync::Parker};
pub use project::{Project, ProjectEnum};

pub use crossbeam_channel as crossbeam;

use crate::search::search_thread;

pub fn run_local(
    paths: impl Iterator<Item = PathBuf>,
    project_filter: Option<Vec<ProjectEnum>>,
) -> crossbeam_channel::Receiver<(std::path::PathBuf, project::ProjectEnum)> {
    let injector = Injector::<PathBuf>::new();

    paths.for_each(|path| {
        injector.push(path);
    });

    let project_filter = project_filter.unwrap_or_else(|| ProjectEnum::ALL.to_vec());

    let thread_count = available_parallelism()
        .unwrap_or(std::num::NonZeroUsize::new(4).unwrap())
        .get();

    let workers = (0..thread_count)
        .map(|_| (Worker::<PathBuf>::new_fifo(), Parker::new()))
        .collect::<Vec<_>>();

    let stealers = workers
        .iter()
        .map(|w| Worker::stealer(&w.0))
        .collect::<Vec<_>>();

    let thread_work_references = workers
        .iter()
        .map(|w| (w.1.unparker().clone(), AtomicCell::new(true)))
        .collect::<Vec<_>>();

    let finished = AtomicCell::new(false);

    // let initial_paths = vec![
    //     PathBuf::from("/Users/choc/wkspaces/Aetherift"), // std::env::current_dir().unwrap()
    // ];

    // for path in initial_paths.into_iter() {
    //     injector.push(path);
    // }

    let worker_thread_idxs = (0..workers.len()).collect::<Vec<_>>();

    let (result_sender, r) = unbounded();

    // let start = std::time::Instant::now();

    std::thread::spawn(move || {
        let senders = (0..thread_count)
            .map(|_| result_sender.clone())
            .collect::<Vec<_>>();

        drop(result_sender);

        std::thread::scope(|s| {
            for (i, (w, p)) in workers.into_iter().enumerate() {
                let active_ref = &thread_work_references[i].1;
                let sender = &senders[i];
                let i = &worker_thread_idxs[i];
                s.spawn(|| {
                    search_thread(
                        i,
                        w,
                        &injector,
                        &stealers,
                        p,
                        active_ref,
                        &finished,
                        sender,
                        &project_filter,
                    )
                });
            }

            loop {
                // Wild guess
                std::thread::sleep(Duration::from_millis(10));

                if thread_work_references
                    .iter()
                    .any(|(_, active)| active.load())
                {
                    let mut sleeping_threads = 0;
                    for (p, active) in thread_work_references.iter() {
                        if !active.load() {
                            sleeping_threads += 1;
                            p.unpark();
                        }
                    }
                    // println!("Sleeping threads: {}", sleeping_threads);
                    // for (i, s) in stealers.iter().enumerate() {
                    // print!("{}={} ", i, s.len());
                    // }
                    // println!();
                } else {
                    finished.store(true);
                    for (p, _) in thread_work_references.iter() {
                        p.unpark();
                    }
                    break;
                }
            }
        });
    });

    // println!("Done Loop");

    // let mut c = 0;
    // for r in r {
    //     c += 1;
    //     // println!("Found project: {} {}", r.1.name(), r.0.display());
    // }

    // println!("Took {}ms", start.elapsed().as_millis());

    // println!("Found {} projects", c);

    r
}

#[cfg(test)]
mod tests {
    use crate::project::Project as _;

    use super::*;

    #[test]
    fn it_works() {
        let initial_paths = vec![
            PathBuf::from("/Users/choc/wkspaces/Aetherift"), // std::env::current_dir().unwrap()
        ];
        // let project_filter = ProjectEnum::ALL;
        let start = std::time::Instant::now();
        let res = run_local(initial_paths.into_iter(), None);
        for r in res {
            println!(
                "Found project: {} {:?} {}",
                r.1.kind_name(),
                r.1.name(&r.0),
                r.0.display()
            );
        }
        println!("Took {}ms", start.elapsed().as_millis());
        assert!(false);
    }
}
