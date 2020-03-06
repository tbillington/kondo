use std::{env, sync::Arc, thread};

use druid::{
    commands::OPEN_FILE,
    widget::{Button, Controller, Flex, Label, List, Scroll, ViewSwitcher, WidgetExt},
    AppLauncher, Command, Data, Env, Event, EventCtx, Lens, LocalizedString, Selector, Widget,
    WindowDesc,
};

use kondo_lib::{clean, pretty_size, scan};

const ADD_ITEM: Selector = Selector::new("event.add-item");
const SET_ACTIVE_ITEM: Selector = Selector::new("event.set-active-item");
const CLEAN_PATH: Selector = Selector::new("event.clean-path");
const SCAN_COMPLETE: Selector = Selector::new("event.scan-complete");

struct EventHandler {}

#[derive(Debug, Clone, Data, Lens)]
struct Project {
    display: String,
    path: String,
    p_type: String,
    artifact_size: u64,
    non_artifact_size: u64,
    dirs: Arc<Vec<(String, u64, bool)>>,
}

impl PartialEq for Project {
    fn eq(&self, other: &Project) -> bool {
        self.path.eq(&other.path)
    }
}

#[derive(Debug, Clone, Data, Lens)]
struct AppData {
    items: Arc<Vec<Project>>,
    active_item: Option<Project>,
    scan_dir: String,
    artifact_size: u64,
    non_artifact_size: u64,
    saved: u64,
    scan_complete: bool,
}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler {}
    }
}

impl<W: Widget<AppData>> Controller<AppData, W> for EventHandler {
    fn event(
        &mut self,
        _child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppData,
        _env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.selector == ADD_ITEM => {
                let project = cmd.get_object::<Project>().unwrap().clone();
                data.artifact_size += project.artifact_size;
                data.non_artifact_size += project.non_artifact_size;
                let items = Arc::make_mut(&mut data.items);
                let pos = items
                    .binary_search_by(|probe| project.artifact_size.cmp(&probe.artifact_size))
                    .unwrap_or_else(|e| e);
                items.insert(pos, project);
                ctx.request_layout();
                ctx.request_paint();
            }
            Event::Command(cmd) if cmd.selector == SET_ACTIVE_ITEM => {
                let active_item = cmd.get_object::<Project>().unwrap().clone();
                data.active_item = Some(active_item);
                ctx.request_layout();
                ctx.request_paint();

                // ctx.submit_command(
                //     Command::new(SHOW_OPEN_PANEL, FileDialogOptions::new()),
                //     None,
                // );
            }
            Event::Command(cmd) if cmd.selector == CLEAN_PATH => {
                let active_item = cmd.get_object::<Project>().unwrap().clone();
                clean(&active_item.path).unwrap();
                data.artifact_size -= active_item.artifact_size;
                data.saved += active_item.artifact_size;
                let items = Arc::make_mut(&mut data.items);
                let pos = items.binary_search_by(|probe| active_item.path.cmp(&probe.path));
                if let Ok(pos) = pos {
                    items.remove(pos);
                }
                data.active_item = None;
                ctx.request_layout();
                ctx.request_paint();
            }
            Event::Command(cmd) if cmd.selector == OPEN_FILE => {
                // let file_info = cmd.get_object::<FileInfo>().unwrap().clone();
                // println!("{:?}", file_info);
            }
            Event::Command(cmd) if cmd.selector == SCAN_COMPLETE => {
                data.scan_complete = true;
                ctx.request_layout();
                ctx.request_paint();
            }
            _ => (),
        }
        _child.event(ctx, event, data, _env);
    }
}

fn main() {
    let window = WindowDesc::new(make_ui)
        .title(LocalizedString::new("kondo-main-window-title").with_placeholder("Kondo 🧹"))
        .window_size((1000.0, 500.0));

    let launcher = AppLauncher::with_window(window);

    let event_sink = launcher.get_external_handle();

    let mut cd = env::current_dir().unwrap();
    cd.pop();
    let scan_dir = String::from(cd.to_str().unwrap());
    let scan_dir_len = scan_dir.len();

    let sd2 = scan_dir.clone();

    thread::spawn(move || {
        scan(&sd2).for_each(|project| {
            let project_size = project.size_dirs();
            let name = project.name();

            let project = Project {
                display: name[scan_dir_len + 1..].to_string(),
                path: name,
                p_type: project.type_name().into(),
                artifact_size: project_size.artifact_size,
                non_artifact_size: project_size.non_artifact_size,
                dirs: Arc::new(project_size.dirs),
            };
            event_sink.submit_command(ADD_ITEM, project, None).unwrap();
        });
        event_sink
            .submit_command(SCAN_COMPLETE, false, None)
            .unwrap();
    });

    launcher
        .use_simple_logger()
        .launch(AppData {
            items: Arc::new(Vec::new()),
            active_item: None,
            scan_dir,
            artifact_size: 0,
            non_artifact_size: 0,
            saved: 0,
            scan_complete: false,
        })
        .expect("launch failed");
}

fn make_ui() -> impl Widget<AppData> {
    let mut root = Flex::column();

    root.add_child(Label::new("Kondo 🧹").padding(10.0).center(), 0.0);
    root.add_child(
        Label::new(|data: &AppData, _env: &_| {
            format!(
                "{} {}",
                data.scan_dir,
                if data.scan_complete {
                    "scan complete ✔️"
                } else {
                    "scan in progress... 📡"
                }
            )
        })
        .center(),
        0.0,
    );

    root.add_child(
        Label::new(|data: &AppData, _env: &_| {
            format!(
                "artifacts {} non-artifacts {} total {} recovered {}",
                pretty_size(data.artifact_size),
                pretty_size(data.non_artifact_size),
                pretty_size(data.artifact_size + data.non_artifact_size),
                pretty_size(data.saved)
            )
        })
        .center(),
        0.0,
    );

    let mut path_listing = Flex::column();
    path_listing.add_child(
        Label::new(|data: &AppData, _env: &_| format!("{} Projects", data.items.len()))
            .padding(10.0)
            .center(),
        0.0,
    );
    let l = Scroll::new(
        List::new(|| {
            Button::new(
                |item: &Project, _env: &_| {
                    format!(
                        "{} ({}) {} / {}",
                        item.display,
                        item.p_type,
                        pretty_size(item.artifact_size),
                        pretty_size(item.artifact_size + item.non_artifact_size)
                    )
                },
                |_ctx, data, _env| {
                    _ctx.submit_command(Command::new(SET_ACTIVE_ITEM, data.clone()), None)
                },
            )
        })
        .lens(AppData::items)
        .padding(2.5),
    )
    .vertical();
    path_listing.add_child(l, 1.0);

    {
        let mut horiz = Flex::row();

        horiz.add_child(path_listing, 1.0);

        {
            let mut vert = Flex::column();
            vert.add_child(
                Label::new("Active Item Information").padding(10.0).center(),
                0.0,
            );
            vert.add_child(
                Label::new(|data: &AppData, _env: &_| match data.active_item {
                    Some(ref project) => format!(
                        "{} {} / {}, {} project",
                        project.display,
                        pretty_size(project.artifact_size),
                        pretty_size(project.artifact_size + project.non_artifact_size),
                        project.p_type
                    ),
                    None => String::from("none selected"),
                }),
                0.0,
            );

            let view_switcher = ViewSwitcher::new(
                |data: &AppData, _env| data.active_item.clone(),
                |selector, _env| match selector {
                    None => Box::new(Label::new("None")),
                    Some(project) => {
                        let project: &Project = project;
                        let mut l = Flex::column();
                        for (i, (dir_name, size, artifact)) in project.dirs.iter().enumerate() {
                            l.add_child(
                                Label::new(format!(
                                    " {}─ {}{} {}",
                                    if i == project.dirs.len() - 1 {
                                        "└"
                                    } else {
                                        "├"
                                    },
                                    dir_name,
                                    if *artifact { "🗑️" } else { "" },
                                    pretty_size(*size)
                                )),
                                0.0,
                            );
                        }
                        Box::new(l)
                    }
                },
            );
            vert.add_child(view_switcher, 0.0);

            vert.add_child(
                Button::new(
                    "Clean project of artifacts",
                    |ctx, data: &mut AppData, _env| {
                        if let Some(active_item) = data.active_item.clone() {
                            ctx.submit_command(Command::new(CLEAN_PATH, active_item), None);
                        }
                    },
                ),
                0.0,
            );

            horiz.add_child(vert.padding(2.5), 1.0);
        }

        root.add_child(horiz, 1.0);
    }

    let cw = EventHandler::new();
    root.controller(cw)
}
