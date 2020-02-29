use std::{env, sync::Arc, thread};

use druid::{
    widget::{Button, Flex, Label, List, Scroll, WidgetExt},
    AppLauncher, BoxConstraints, Command, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle,
    LifeCycleCtx, LocalizedString, PaintCtx, Selector, Size, UpdateCtx, Widget, WindowDesc,
};

use kondo_lib::{pretty_size, scan};

const ADD_ITEM: Selector = Selector::new("event.add-item");
const SET_ACTIVE_ITEM: Selector = Selector::new("event.set-active-item");

struct EventHandler {}

type ItemData = (String, u64);

#[derive(Debug, Clone, Data, Lens)]
struct AppData {
    items: Arc<Vec<ItemData>>,
    active_item: Option<ItemData>,
    scan_dir: String,
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
                let items = Arc::make_mut(&mut data.items);
                let pos = items
                    .binary_search_by(|probe| new_elem.1.cmp(&probe.1))
                    .unwrap_or_else(|e| e);
                items.insert(pos, new_elem);
                ctx.request_paint();
            }
            Event::Command(cmd) if cmd.selector == SET_ACTIVE_ITEM => {
                let active_string = cmd.get_object::<String>().unwrap().clone();
                data.active_item = Some((active_string, 10));
                ctx.request_paint();
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
        .title(LocalizedString::new("kondo-main-window-title").with_placeholder("Kondo"));

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
                "total: {}",
                pretty_size(data.items.iter().map(|x| x.1).sum())
            )
        })
        .center(),
        0.0,
    );

    let l = Scroll::new(
        List::new(|| {
            Button::new(
                |item: &ItemData, _env: &_| format!("{}: {}", item.0, pretty_size(item.1)),
                |_ctx, _data, _env| {
                    _ctx.submit_command(Command::new(SET_ACTIVE_ITEM, _data.0.clone()), None)
                },
            )
        })
        .lens(AppData::items),
    )
    .vertical();

    {
        let mut horiz = Flex::row();

        horiz.add_child(l, 1.0);

        {
            let mut vert = Flex::column();
            vert.add_child(Label::new("Active Item Information"), 0.0);
            vert.add_child(
                Label::new(|data: &AppData, _env: &_| match data.active_item {
                    Some((ref name, size)) => format!("{} {}", name, size),
                    None => String::from("none selected"),
                }),
                0.0,
            );
            horiz.add_child(vert, 1.0);
        }

        root.add_child(horiz, 1.0);
    }

    let cw = EventHandler::new().fix_width(0.0).fix_height(0.0);

    root.add_child(cw, 0.0);
    root
}
