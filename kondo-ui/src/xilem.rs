use std::{cmp::Ordering, convert::identity, path::Path, sync::mpsc, thread};

use kondo_lib::{pretty_size, scan, ScanOptions};
use winit::{dpi::LogicalSize, window::Window};
use xilem::{
    core::{fork, one_of::Either, MessageProxy, PhantomView},
    view::{
        button, flex, grid, label, portal, prose, sized_box, textbox, worker_raw, Axis,
        CrossAxisAlignment, FlexExt, GridExt,
    },
    Color, TextAlignment, ViewCtx, WidgetView, Xilem,
};

fn spawn_scanner_thread(
    scan_starter_recv: mpsc::Receiver<ScanStarterThreadMsg>,
    options: ScanOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    thread::Builder::new()
        .name(String::from("scan"))
        .spawn(move || {
            while let Ok(ScanStarterThreadMsg::StartScan(path, proxy)) = scan_starter_recv.recv() {
                if !std::fs::exists(&path).unwrap_or(true) {
                    let message_result = proxy.message(ScanResult::InvalidPath);
                    if message_result.is_err() {
                        // The corresponding `View` has been deleted, wait for the next task
                        // TODO(Xilem side): That's not actually true
                        continue;
                    }
                    continue;
                }
                for project in scan(&path, &options).filter_map(|p| p.ok()) {
                    let name = project.name().to_string();
                    let project_size = project.size_dirs(&options);
                    let display = Path::new(&name)
                        .file_name()
                        .map(|s| s.to_str().unwrap_or(&name))
                        .unwrap_or(&name);
                    let project = Project {
                        display: String::from(display),
                        path: name,
                        p_type: project.type_name().into(),
                        artifact_size: project_size.artifact_size,
                        non_artifact_size: project_size.non_artifact_size,
                        dirs: project_size.dirs,
                    };
                    if proxy.message(ScanResult::AddItem(project)).is_err() {
                        continue;
                    }
                }
                if proxy.message(ScanResult::Complete).is_err() {
                    continue;
                }
            }
        })?;
    Ok(())
}

#[derive(Debug, Clone)]
struct Project {
    display: String,
    path: String,
    p_type: String,
    artifact_size: u64,
    non_artifact_size: u64,
    dirs: Vec<(String, u64, bool)>,
}

impl PartialEq for Project {
    fn eq(&self, other: &Project) -> bool {
        self.path.eq(&other.path)
    }
}

impl Ord for Project {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.artifact_size.cmp(&other.artifact_size) {
            Ordering::Equal => self.display.cmp(&other.display),
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        }
    }
}

impl PartialOrd for Project {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Project {}

#[derive(Debug, Clone, PartialEq)]
enum ScanStatus {
    NotStarted,
    InProgress,
    InvalidPath,
    Complete,
}

enum ScanStarterThreadMsg {
    StartScan(String, MessageProxy<ScanResult>),
}

#[derive(Debug)]
enum ScanResult {
    AddItem(Project),
    Complete,
    InvalidPath,
}

struct Kondo {
    items: Vec<Project>,
    active_item: Option<Project>,
    scan_dir: String,
    scan_dir_input: String,
    artifact_size: u64,
    non_artifact_size: u64,
    saved: u64,
    scan_complete: ScanStatus,
    scan_starter_send: mpsc::SyncSender<ScanStarterThreadMsg>,
}

fn scanner(data: &mut Kondo) -> impl PhantomView<Kondo, (), ViewCtx> {
    let sender = data.scan_starter_send.clone();
    // We use the raw version here, because we do need to move a (clone of)
    // sender into the worker
    worker_raw(
        data.scan_dir.to_string(),
        move |proxy, mut messages| {
            let sender = sender.clone();
            async move {
                while let Some(message) = messages.recv().await {
                    if message.is_empty() {
                        continue;
                    }
                    match sender.send(ScanStarterThreadMsg::StartScan(message, proxy.clone())) {
                        Ok(()) => {}
                        Err(_) => break,
                    };
                }
            }
        },
        |data: &mut Kondo, response: ScanResult| match response {
            ScanResult::AddItem(project) => {
                data.artifact_size += project.artifact_size;
                data.non_artifact_size += project.non_artifact_size;
                let pos = data.items.binary_search(&project).unwrap_or_else(identity);
                data.items.insert(pos, project);
            }
            ScanResult::Complete => data.scan_complete = ScanStatus::Complete,
            ScanResult::InvalidPath => data.scan_complete = ScanStatus::InvalidPath,
        },
    )
}

impl Kondo {
    fn view(&mut self) -> impl WidgetView<Self> {
        let header = (
            prose("Kondo 🧹")
                .alignment(TextAlignment::Middle)
                .brush(Color::rgb(0.5, 0.75, 1.0))
                .flex(CrossAxisAlignment::Center),
            textbox(
                self.scan_dir_input.to_string(),
                |data: &mut Kondo, new_content| data.scan_dir_input = new_content,
            )
            .on_enter(|data, result| {
                // TODO: Does on_enter imply on_changed?
                data.scan_dir_input = result;
                data.update_scan_target();
            }),
            flex((
                prose(format!(
                    "{} {}",
                    self.scan_dir,
                    match self.scan_complete {
                        ScanStatus::Complete => "scan complete ✔️",
                        ScanStatus::InProgress => "scan in progress... 📡",
                        ScanStatus::NotStarted => "scan not started",
                        ScanStatus::InvalidPath => "scan cancelled due to invalid path",
                    }
                )),
                button("Select Directory", |data: &mut Kondo| {
                    data.update_scan_target();
                }),
            ))
            .direction(Axis::Horizontal)
            .flex(CrossAxisAlignment::Center),
            prose(format!(
                "artifacts {}\n\
                non-artifacts {}\n\
                total {}\n\
                recovered {}",
                pretty_size(self.artifact_size),
                pretty_size(self.non_artifact_size),
                pretty_size(self.artifact_size + self.non_artifact_size),
                pretty_size(self.saved)
            ))
            .alignment(TextAlignment::Middle)
            .flex(CrossAxisAlignment::Fill),
        );
        let path_listing = flex((
            prose(format!("{} projects", self.items.len())).alignment(TextAlignment::Middle),
            portal(
                flex((
                    label("vello#644 workaround").text_size(0.1),
                    self.items
                        .iter()
                        .enumerate()
                        .map(|(idx, item)| {
                            sized_box(button(
                                format!(
                                    "{} ({}) {} / {}",
                                    item.display,
                                    item.p_type,
                                    pretty_size(item.artifact_size),
                                    pretty_size(item.artifact_size + item.non_artifact_size)
                                ),
                                move |data: &mut Kondo| {
                                    data.active_item = Some(data.items[idx].clone());
                                },
                            ))
                            .expand_width()
                        })
                        .collect::<Vec<_>>(),
                ))
                .cross_axis_alignment(CrossAxisAlignment::Fill)
                .gap(5.),
            ),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Fill);
        let maybe_active_item = match self.active_item.as_ref() {
            None => Either::A(prose(
                "No project selected. Choose a project from the list to the left.",
            )),
            Some(project) => Either::B(
                flex((
                    prose(project.display.as_str()),
                    project
                        .dirs
                        .iter()
                        .enumerate()
                        .map(|(i, (dir_name, size, artifact))| {
                            prose(format!(
                                " {}─ {}{} {}",
                                if i == project.dirs.len() - 1 {
                                    "└"
                                } else {
                                    "├"
                                },
                                dir_name,
                                if *artifact {
                                    /* "🗑️" */
                                    "(artifact)"
                                } else {
                                    ""
                                },
                                pretty_size(*size)
                            ))
                        })
                        .collect::<Vec<_>>(),
                ))
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .gap(0.0),
            ),
        };

        let scanner = scanner(self);
        fork(
            flex((
                header,
                // Use a Grid to force a 1x2 layout, because for reasons
                // unknown this doesn't work with flex
                grid(
                    (
                        path_listing.grid_pos(0, 0),
                        maybe_active_item.grid_pos(1, 0),
                    ),
                    2,
                    1,
                )
                .flex(CrossAxisAlignment::Fill),
            ))
            .must_fill_major_axis(true),
            scanner,
        )
    }

    fn update_scan_target(&mut self) {
        // TODO: Use a file select dialogue instead
        self.scan_dir = self.scan_dir_input.clone();

        self.active_item = None;
        self.artifact_size = 0;
        self.items.clear();
        self.non_artifact_size = 0;
        self.scan_complete = ScanStatus::InProgress;
    }
}

pub(crate) fn xilem_main() {
    let attrs = Window::default_attributes()
        .with_title("Kondo 🧹")
        .with_inner_size(LogicalSize::new(1000., 500.));

    let (scan_starter_send, scan_starter_recv) = mpsc::sync_channel::<ScanStarterThreadMsg>(0);

    let scan_options = ScanOptions {
        follow_symlinks: false,
        same_file_system: true,
    };

    spawn_scanner_thread(scan_starter_recv, scan_options).expect("error spawning scan thread");
    let home_dir = homedir::my_home()
        .ok()
        .flatten()
        .and_then(|it| it.into_os_string().into_string().ok());

    let kondo = Kondo {
        items: Vec::new(),
        active_item: None,
        scan_dir: String::new(),
        scan_dir_input: home_dir.unwrap_or_default(),
        artifact_size: 0,
        non_artifact_size: 0,
        saved: 0,
        scan_complete: ScanStatus::NotStarted,
        scan_starter_send,
    };
    let event_loop = winit::event_loop::EventLoop::with_user_event();

    Xilem::new(kondo, Kondo::view)
        .run_windowed_in(event_loop, attrs)
        .unwrap();
}
