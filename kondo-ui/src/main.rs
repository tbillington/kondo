use std::{env, sync::Arc, thread};

use druid::{
    commands::{OPEN_FILE, SHOW_OPEN_PANEL},
    widget::{Button, Container, Flex, Label, List, Scroll, WidgetExt},
    AppLauncher, BoxConstraints, Color, Command, Data, Env, Event, EventCtx, FileDialogOptions,
    FileInfo, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, LocalizedString, PaintCtx, Selector, Size,
    UpdateCtx, Widget, WindowDesc,
};

use kondo_lib::{clean, pretty_size, scan};

const ADD_ITEM: Selector = Selector::new("event.add-item");
const SET_ACTIVE_ITEM: Selector = Selector::new("event.set-active-item");
const CLEAN_PATH: Selector = Selector::new("event.clean-path");

struct EventHandler {}

type ItemData = (String, u64);

#[derive(Debug, Clone, Data, Lens)]
struct AppData {
    items: Arc<Vec<ItemData>>,
    active_item: Option<ItemData>,
    scan_dir: String,
    total: u64,
    saved: u64,
}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler {}
    }
}

impl Widget<AppData> for EventHandler {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            Event::Command(cmd) if cmd.selector == ADD_ITEM => {
                let new_elem = cmd.get_object::<ItemData>().unwrap().clone();
                data.total += new_elem.1;
                let items = Arc::make_mut(&mut data.items);
                let pos = items
                    .binary_search_by(|probe| new_elem.1.cmp(&probe.1))
                    .unwrap_or_else(|e| e);
                items.insert(pos, new_elem);
                ctx.request_paint();
            }
            Event::Command(cmd) if cmd.selector == SET_ACTIVE_ITEM => {
                let active_item = cmd.get_object::<ItemData>().unwrap().clone();
                data.active_item = Some(active_item);
                ctx.request_paint();

                // ctx.submit_command(
                //     Command::new(SHOW_OPEN_PANEL, FileDialogOptions::new()),
                //     None,
                // );
            }
            Event::Command(cmd) if cmd.selector == CLEAN_PATH => {
                let active_item = cmd.get_object::<ItemData>().unwrap().clone();
                clean(&active_item.0).unwrap();
                data.total -= active_item.1;
                data.saved += active_item.1;
                let items = Arc::make_mut(&mut data.items);
                let pos = items.binary_search_by(|probe| active_item.1.cmp(&probe.1));
                if let Ok(pos) = pos {
                    items.remove(pos);
                }
                data.active_item = None;
                ctx.request_paint();
            }
            Event::Command(cmd) if cmd.selector == OPEN_FILE => {
                // let file_info = cmd.get_object::<FileInfo>().unwrap().clone();
                // println!("{:?}", file_info);
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &AppData, _: &Env) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppData, data: &AppData, _: &Env) {
        if !old_data.same(data) {
            ctx.request_paint()
        }
    }

    fn layout(&mut self, _: &mut LayoutCtx, bc: &BoxConstraints, _: &AppData, _: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, _ctx: &mut PaintCtx, _data: &AppData, _env: &Env) {}
}

fn main() {
    let window = WindowDesc::new(make_ui)
        .title(LocalizedString::new("kondo-main-window-title").with_placeholder("Kondo"))
        .window_size((1000.0, 500.0));

    let launcher = AppLauncher::with_window(window);

    let event_sink = launcher.get_external_handle();

    let scan_dir = String::from(env::current_dir().unwrap().to_str().unwrap());

    let sd2 = scan_dir.clone();

    thread::spawn(move || {
        scan(&sd2)
            .filter_map(|p| match p.size() {
                0 => None,
                size => Some((p.name(), size)),
            })
            .for_each(|p| {
                event_sink.submit_command(ADD_ITEM, p, None).unwrap();
            });
    });

    launcher
        .use_simple_logger()
        .launch(AppData {
            items: Arc::new(vec![]),
            active_item: None,
            scan_dir,
            total: 0,
            saved: 0,
        })
        .expect("launch failed");
}

fn make_ui() -> impl Widget<AppData> {
    let mut root = Flex::column();

    root.add_child(Label::new("Kondo").padding(10.0).center(), 0.0);
    root.add_child(
        Label::new(|data: &AppData, _env: &_| format!("scanning {}", data.scan_dir)).center(),
        0.0,
    );

    root.add_child(
        Label::new(|data: &AppData, _env: &_| {
            format!(
                "total: {} recovered: {}",
                pretty_size(data.total),
                pretty_size(data.saved)
            )
        })
        .center(),
        0.0,
    );

    let mut path_listing = Flex::column();
    path_listing.add_child(Label::new("Projects").padding(10.0).center(), 0.0);
    let l = Scroll::new(
        List::new(|| {
            Button::new(
                |item: &ItemData, _env: &_| format!("{}: {}", item.0, pretty_size(item.1)),
                |_ctx, data, _env| {
                    _ctx.submit_command(
                        Command::new(SET_ACTIVE_ITEM, (data.0.clone(), data.1)),
                        None,
                    )
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
                    Some((ref name, size)) => format!("{} {}", name, pretty_size(size)),
                    None => String::from("none selected"),
                }),
                0.0,
            );
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

            horiz.add_child(
                // Container::new(vert.padding(2.5)).border(Color::rgb(1.0, 0.0, 0.0), 2.0),
                vert.padding(2.5),
                1.0,
            );
        }

        root.add_child(horiz, 1.0);
    }

    let cw = EventHandler::new().fix_width(0.0).fix_height(0.0);

    root.add_child(cw, 0.0);
    root
}
