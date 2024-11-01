use std::{
    cmp::Ordering,
    convert::identity,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
};

use kondo_lib::{pretty_size, scan, ScanOptions};
use winit::{dpi::LogicalSize, window::Window};
use xilem::{
    core::{fork, MessageProxy, PhantomView},
    view::{
        button, flex, label, portal, prose, textbox, worker, worker_raw, Axis, Button,
        CrossAxisAlignment, FlexExt, FlexParams, Portal,
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
            while let Ok(ScanStarterThreadMsg::StartScan(p, proxy)) = scan_starter_recv.recv() {
                for project in scan(&p, &options).filter_map(|p| p.ok()) {
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
                        // The corresponding `View` has been deleted
                        // TODO: That's not actually true
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
    Complete,
}

enum ScanStarterThreadMsg {
    StartScan(String, MessageProxy<ScanResult>),
}

#[derive(Debug)]
enum ScanResult {
    AddItem(Project),
    Complete,
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
        },
    )
}

impl Kondo {
    fn view(&mut self) -> impl WidgetView<Self> {
        let header = (
            label("Kondo 🧹")
                .alignment(TextAlignment::Middle)
                .brush(Color::rgb(0.5, 0.75, 1.0))
                .flex(CrossAxisAlignment::Center),
            textbox(
                self.scan_dir_input.to_string(),
                |data: &mut Kondo, new_content| data.scan_dir_input = new_content,
            ),
            flex((
                label(format!(
                    "{} {}",
                    self.scan_dir,
                    match self.scan_complete {
                        ScanStatus::Complete => "scan complete ✔️",
                        ScanStatus::InProgress => "scan in progress... 📡",
                        ScanStatus::NotStarted => "scan not started",
                    }
                )),
                button("Select Directory", |data: &mut Kondo| {
                    // TODO: Use a file select dialogue instead
                    data.scan_dir = data.scan_dir_input.clone();

                    data.active_item = None;
                    data.artifact_size = 0;
                    data.items.clear();
                    data.non_artifact_size = 0;
                    data.scan_complete = ScanStatus::InProgress;
                }),
            ))
            .direction(Axis::Horizontal)
            .flex(CrossAxisAlignment::Center),
            prose(format!(
                "artifacts {} non-artifacts {} total {} recovered {}",
                pretty_size(self.artifact_size),
                pretty_size(self.non_artifact_size),
                pretty_size(self.artifact_size + self.non_artifact_size),
                pretty_size(self.saved)
            ))
            .alignment(TextAlignment::Middle)
            .flex(CrossAxisAlignment::Center),
        );
        let path_listing = flex((
            prose(format!("{} projects", self.items.len()))
                .alignment(TextAlignment::Middle)
                .flex(FlexParams::new(1.0, CrossAxisAlignment::Center)),
            portal(
                flex(
                    self.items
                        .iter()
                        .enumerate()
                        .map(|(idx, item)| {
                            button(
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
                            )
                        })
                        .collect::<Vec<_>>(),
                )
                .gap(5.),
            ),
        ));
        let vert = flex(());

        let scanner = scanner(self);
        fork(
            flex((
                header,
                flex((path_listing.flex(1.0), vert.flex(1.0)))
                    .direction(Axis::Horizontal)
                    .must_fill_major_axis(true),
            )),
            scanner,
        )
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
