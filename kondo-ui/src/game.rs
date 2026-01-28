use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};

use bevy::{
    ecs::system::RunSystemOnce,
    feathers::{
        FeathersPlugin,
        controls::{ButtonProps, ButtonVariant, button},
        dark_theme::create_dark_theme,
        theme::{ThemeBackgroundColor, ThemedText, UiTheme},
        tokens,
    },
    input_focus::{
        InputDispatchPlugin,
        tab_navigation::{TabGroup, TabNavigationPlugin},
    },
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
    ui_widgets::{Activate, UiWidgetsPlugins, observe},
};

pub(super) fn game_plugin(app: &mut App) {
    app.set_error_handler(bevy::ecs::error::error);

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Kondo 🧹".to_owned(),
            ..default()
        }),
        ..default()
    }));

    app.add_plugins((
        UiWidgetsPlugins,
        InputDispatchPlugin,
        TabNavigationPlugin,
        FeathersPlugin,
    ));

    // {
    //     use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
    //     app.add_plugins(EguiPlugin::default());
    //     app.add_plugins(WorldInspectorPlugin::new());
    // }

    app.insert_resource(UiTheme(create_dark_theme()));

    app.add_systems(Startup, (setup, spawn_project_list).chain());

    app.add_systems(
        Update,
        (
            process_new_projects,
            update_project_list_ui,
            select_project_update,
            handle_clean_tasks,
        )
            .chain(),
    );

    app.add_systems(Update, send_scroll_events);

    app.add_observer(on_scroll_handler);

    app.insert_resource(SelectedProject(None));

    app.insert_non_send_resource(BackgroundThreadCommunication::default());
}

#[derive(Component)]
struct CleanTask(Task<kondo_lib::Project>);

fn handle_clean_tasks(
    mut clean_tasks: Query<(Entity, &mut CleanTask)>,
    mut pl: ResMut<ProjectList>,
    mut sp: ResMut<SelectedProject>,
    mut c: Commands,
) {
    for (e, mut task) in &mut clean_tasks {
        if let Some(proj) = check_ready(&mut task.0) {
            {
                let proj = pl.0.iter_mut().find(|p| p.kproj.path == proj.path);

                if let Some(proj) = proj {
                    proj.status = ProjectListEntryStatus::Cleaned;
                    proj.size = 0;
                }
            }

            c.entity(e).despawn();

            if let Some(ple) = &sp.0
                && ple.kproj.path == proj.path
            {
                sp.0 = None;
            }
        }
    }
}

enum BackgroundThreadMsg {
    ScanningStarted(Vec<PathBuf>),
    PLE(ProjectListEntry),
    #[expect(unused)]
    ScanningFinished,
}

struct BackgroundThreadCommunication {
    send: Sender<BackgroundThreadMsg>,
    recv: Receiver<BackgroundThreadMsg>,
}

impl Default for BackgroundThreadCommunication {
    fn default() -> Self {
        let (send, recv) = std::sync::mpsc::channel();

        Self { send, recv }
    }
}

const SCAN_OPTIONS: &kondo_lib::ScanOptions = &kondo_lib::ScanOptions {
    follow_symlinks: false,
    same_file_system: false,
};

fn setup(mut c: Commands) {
    c.spawn(Camera2d);

    let root = spawn_root(&mut c);

    let _root_id = c.spawn(root).id();
}

#[derive(Resource, Deref, DerefMut)]
struct ProjectList(Vec<ProjectListEntry>);

fn spawn_project_list(
    root_ui: Single<Entity, With<RootUITag>>,
    pl: Query<Entity, With<ProjectListTag>>,
    mut c: Commands,
) {
    for pl in pl.iter() {
        c.entity(pl).despawn();
    }

    let ples = vec![];

    c.insert_resource(ProjectList(ples.clone()));

    let central_area_ui = c
        .spawn((
            CentralUIAreaTag,
            ChildOf(*root_ui),
            Node {
                flex_direction: FlexDirection::Row,
                // row_gap: Val::Px(8.),
                // padding: Val::Px(8.).into(),
                // overflow: Overflow::scroll_y(),
                // height: Val::Percent(100.),
                width: Val::Percent(100.),
                min_height: Val::Px(0.),
                // flex_shrink: 0.,
                ..default()
            },
        ))
        .id();

    let left_side = c
        .spawn((
            LeftSideTag,
            ChildOf(central_area_ui),
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(50.),
                ..default()
            },
        ))
        .id();

    c.spawn((
        ScanningDirsListTag,
        ChildOf(left_side),
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.),
            padding: Val::Px(8.).into(),
            // overflow: Overflow::scroll_y(),
            width: Val::Percent(100.),
            ..default()
        },
    ));

    c.spawn((
        ProjectListTag,
        ChildOf(left_side),
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.),
            padding: Val::Px(8.).into(),
            overflow: Overflow::scroll_y(),
            width: Val::Percent(100.),
            // height: Val::Percent(100.),
            // flex_shrink: 0.,
            ..default()
        },
    ))
    .with_children(|c| {
        for ple in ples {
            c.spawn(build_project_list_entry(ple));
        }
    });

    c.spawn((
        SelectedProjectTag,
        ChildOf(central_area_ui),
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.),
            padding: Val::Px(8.).into(),
            overflow: Overflow::scroll_y(),
            width: Val::Percent(50.),
            // flex_shrink: 0.,
            ..default()
        },
    ))
    .with_children(|c| {
        c.spawn((Text::new("Selected Project"), ThemedText));
    });
}

fn project_file_name(proj: &kondo_lib::Project) -> String {
    proj.path
        .file_name()
        .map(|n| n.to_string_lossy())
        // fallback to filepath which is kind of shitty
        .unwrap_or_else(|| proj.name())
        .to_string()
}

fn select_project_update(
    root: Query<Entity, With<SelectedProjectTag>>,
    sp: Res<SelectedProject>,
    mut c: Commands,
) {
    let Ok(root) = root.single() else {
        return;
    };

    if !sp.is_changed() {
        return;
    }

    c.entity(root).despawn_children();

    let Some(ple) = &sp.0 else {
        return;
    };

    let proj = ple.kproj.clone();

    let display_name = project_file_name(&ple.kproj);

    let mut dir_sizes = ple.kproj.size_dirs(SCAN_OPTIONS);

    dir_sizes
        .dirs
        .sort_unstable_by_key(|d| std::cmp::Reverse(d.1));

    c.spawn((
        ChildOf(root),
        Node {
            width: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.),
            padding: UiRect::horizontal(Val::Px(16.)),
            ..default()
        },
        font(16.),
        Children::spawn((
            Spawn((
                Node {
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                font(20.),
                Children::spawn_one((Text::new(display_name), ThemedText)),
            )),
            Spawn((Text::new(ple.kproj.path.to_string_lossy()), ThemedText)),
            Spawn((
                Text::new(format!("{} project", ple.kproj.type_name())),
                ThemedText,
            )),
            Spawn((
                Text::new(format!(
                    "{} Total Size",
                    kondo_lib::pretty_size(dir_sizes.artifact_size + dir_sizes.non_artifact_size)
                )),
                ThemedText,
            )),
            Spawn((
                Text::new(format!(
                    "{} Artifact Size",
                    kondo_lib::pretty_size(dir_sizes.artifact_size)
                )),
                ThemedText,
            )),
            Spawn((
                Node {
                    padding: UiRect::left(Val::Px(32.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.),
                    ..default()
                },
                ThemedText,
                Children::spawn(SpawnIter(dir_sizes.dirs.into_iter().map(
                    |(name, size, artifacts)| {
                        (
                            Text::new(format!(
                                "{}{}{}",
                                name,
                                if artifacts {
                                    " (Artifact) " /* 🗑️ */
                                } else {
                                    " "
                                },
                                kondo_lib::pretty_size(size)
                            )),
                            ThemedText,
                        )
                    },
                ))),
            )),
            Spawn((
                button(
                    ButtonProps {
                        variant: ButtonVariant::Primary,
                        ..default()
                    },
                    (),
                    Spawn((
                        // font(16.),
                        Text::new("Delete Artifacts"),
                        ThemedText,
                    )),
                ),
                observe(
                    move |_: On<Activate>, mut pl: ResMut<ProjectList>, mut c: Commands| {
                        let proj = proj.clone();

                        let ple = pl.0.iter_mut().find(|p| p.kproj.path == proj.path);

                        if let Some(ple) = ple {
                            ple.status = ProjectListEntryStatus::Cleaning;
                        }

                        let thread_pool = AsyncComputeTaskPool::get();
                        let task = thread_pool.spawn(async move {
                            let start = std::time::Instant::now();
                            proj.clean();
                            let elapsed = start.elapsed();

                            info!("Cleaned {:?} in {}ms", &proj, elapsed.as_millis());

                            proj
                        });

                        c.spawn(CleanTask(task));
                    },
                ),
            )),
        )),
    ));
}

fn font(size: f32) -> bevy::feathers::font_styles::InheritableFont {
    bevy::feathers::font_styles::InheritableFont {
        font: bevy::feathers::handle_or_path::HandleOrPath::Path(
            bevy::feathers::constants::fonts::REGULAR.to_owned(),
        ),
        font_size: size,
    }
}

fn update_project_list_ui(
    q: Query<Entity, With<ProjectListTag>>,
    pl: Res<ProjectList>,
    mut c: Commands,
) {
    if !pl.is_changed() {
        return;
    }

    for pl_ui in q.iter() {
        c.entity(pl_ui).despawn_children().with_children(|c| {
            for p in pl.iter() {
                c.spawn(build_project_list_entry(p.clone()));
            }
        });
    }

    c.queue(SortProjectList::Size);
}

#[derive(Component)]
struct RootUITag;

fn discover_projects<'a>(
    dirs: &'a Vec<std::path::PathBuf>,
) -> impl Iterator<Item = kondo_lib::Project> {
    dirs.iter()
        .flatten()
        .into_iter()
        .flat_map(|dir| kondo_lib::scan(&dir, SCAN_OPTIONS))
        .filter_map(Result::ok)
}

fn process_new_projects(
    tc: NonSend<BackgroundThreadCommunication>,
    mut pl: ResMut<ProjectList>,
    sdl: Query<Entity, With<ScanningDirsListTag>>,
    mut c: Commands,
) {
    while let Ok(msg) = tc.recv.try_recv() {
        match msg {
            BackgroundThreadMsg::ScanningStarted(dirs) => {
                pl.clear();

                if let Some(sdl) = sdl.iter().next() {
                    c.entity(sdl).despawn_children();
                    for dir in dirs {
                        c.spawn((
                            Text::new(dir.to_string_lossy().to_owned()),
                            ThemedText,
                            ChildOf(sdl),
                        ));
                    }
                }
            }
            BackgroundThreadMsg::PLE(ple) => {
                pl.push(ple);
            }
            BackgroundThreadMsg::ScanningFinished => {}
        }
    }
}

fn select_directory(_: On<Activate>, tc: NonSend<BackgroundThreadCommunication>) {
    let main_thread_send = tc.send.clone();

    std::thread::spawn(move || {
        let Some(dirs) = rfd::FileDialog::new().pick_folders() else {
            return;
        };

        if main_thread_send
            .send(BackgroundThreadMsg::ScanningStarted(dirs.clone()))
            .is_err()
        {
            return;
        }

        let (raw_proj_send, raw_proj_recv) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            info!("Searching {:?}", &dirs);
            for raw_proj in discover_projects(&dirs) {
                if raw_proj_send.send(raw_proj).is_err() {
                    return;
                }
            }
        });

        while let Ok(raw_proj) = raw_proj_recv.recv() {
            let proj_entry = ProjectListEntry {
                size: raw_proj.size(SCAN_OPTIONS),
                kproj: raw_proj,
                status: ProjectListEntryStatus::Uncleaned,
            };

            if main_thread_send
                .send(BackgroundThreadMsg::PLE(proj_entry))
                .is_err()
            {
                return;
            }
        }
    });
}

fn spawn_root(_: &mut Commands) -> impl Bundle {
    (
        RootUITag,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        },
        TabGroup::default(),
        ThemeBackgroundColor(tokens::WINDOW_BG),
        Children::spawn_one((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                column_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(8.)),
                ..default()
            },
            Children::spawn((
                Spawn((
                    button(
                        ButtonProps::default(),
                        (),
                        Spawn((Text::new("Sort by Name"), ThemedText)),
                    ),
                    observe(|_: On<Activate>, mut c: Commands| c.queue(SortProjectList::Name)),
                )),
                Spawn((
                    button(
                        ButtonProps::default(),
                        (),
                        Spawn((Text::new("Sort by Size"), ThemedText)),
                    ),
                    observe(|_: On<Activate>, mut c: Commands| c.queue(SortProjectList::Size)),
                )),
                Spawn((
                    button(
                        ButtonProps {
                            variant: ButtonVariant::Primary,
                            ..default()
                        },
                        (),
                        Spawn((Text::new("Select Directory"), ThemedText)),
                    ),
                    observe(select_directory),
                )),
            )),
        )),
    )
}

#[derive(Component)]
struct LeftSideTag;

#[derive(Component)]
struct ScanningDirsListTag;

#[derive(Component)]
struct ProjectListTag;

#[derive(Component)]
struct SelectedProjectTag;

#[derive(Component)]
struct CentralUIAreaTag;

#[derive(Component, Clone)]
struct ProjectListEntry {
    kproj: kondo_lib::Project,
    size: u64,
    status: ProjectListEntryStatus,
}

#[derive(Clone)]
enum ProjectListEntryStatus {
    Uncleaned,
    Cleaning,
    Cleaned,
}

fn build_project_list_entry(ple: ProjectListEntry) -> impl Bundle {
    let proj = &ple.kproj;
    let display_name = proj
        .path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_else(|| proj.name());
    let text = format!(
        "{} ({}) {} {}",
        display_name,
        proj.type_name(),
        kondo_lib::pretty_size(ple.size),
        match ple.status {
            ProjectListEntryStatus::Uncleaned => "",
            ProjectListEntryStatus::Cleaning => "Cleaning",
            ProjectListEntryStatus::Cleaned => "Cleaned",
        }
    );

    (
        Node {
            // padding: UiRect::vertical(Val::Px(8.0)),
            ..default()
        },
        ple.clone(),
        Children::spawn_one((
            button_left(
                ButtonProps::default(),
                ple,
                Spawn((
                    Text::new(text),
                    TextLayout::new_with_linebreak(LineBreak::WordOrCharacter),
                    ThemedText,
                )),
            ),
            observe(project_list_entry_clicked),
        )),
    )
}

pub fn button_left<
    C: bevy::ecs::spawn::SpawnableList<ChildOf> + Send + Sync + 'static,
    B: Bundle,
>(
    props: ButtonProps,
    overrides: B,
    children: C,
) -> impl Bundle {
    (
        Node {
            // height: bevy::feathers::constants::size::ROW_HEIGHT,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.)),
            flex_grow: 1.0,
            border_radius: props.corners.to_border_radius(4.0),
            ..Default::default()
        },
        bevy::ui_widgets::Button,
        props.variant,
        bevy::picking::hover::Hovered::default(),
        bevy::feathers::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Pointer),
        bevy::input_focus::tab_navigation::TabIndex(0),
        ThemeBackgroundColor(tokens::BUTTON_BG),
        bevy::feathers::theme::ThemeFontColor(tokens::BUTTON_TEXT),
        bevy::feathers::font_styles::InheritableFont {
            font: bevy::feathers::handle_or_path::HandleOrPath::Path(
                bevy::feathers::constants::fonts::REGULAR.to_owned(),
            ),
            font_size: 14.0,
        },
        overrides,
        Children::spawn(children),
    )
}

#[derive(Resource)]
struct SelectedProject(Option<ProjectListEntry>);

fn project_list_entry_clicked(
    on: On<Activate>,
    ple: Query<&ProjectListEntry>,
    mut sp: ResMut<SelectedProject>,
) {
    let ple_id = on.event_target();
    if let Ok(ple) = ple.get(ple_id) {
        sp.0 = Some(ple.clone());
    }
}

enum SortProjectList {
    Name,
    Size,
}

impl Command for SortProjectList {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(sort_projectlist, self).unwrap();
    }
}

fn sort_projectlist(
    In(sort): In<SortProjectList>,
    mut pl: Query<&mut Children, With<ProjectListTag>>,
    ple: Query<&ProjectListEntry>,
) {
    for mut pl in pl.iter_mut() {
        match sort {
            SortProjectList::Name => {
                pl.sort_by_key(|k| {
                    let key = project_file_name(&ple.get(*k).unwrap().kproj);
                    key
                });
            }
            SortProjectList::Size => {
                pl.sort_by_key(|k| {
                    let ple = ple.get(*k).unwrap();
                    let key = std::cmp::Reverse(ple.size);
                    key
                });
            }
        }
    }
}

const LINE_HEIGHT: f32 = 21.;

// Copied from Bevy feathers example
/// Injects scroll events into the UI hierarchy.
fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<bevy::input::mouse::MouseWheel>,
    hover_map: Res<bevy::picking::hover::HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == bevy::input::mouse::MouseScrollUnit::Line {
            delta *= LINE_HEIGHT;
        }

        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
}

// Copied from Bevy feathers example
/// UI scrolling event.
#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
struct Scroll {
    entity: Entity,
    /// Scroll delta in logical coordinates.
    delta: Vec2,
}

// Copied from Bevy feathers example
fn on_scroll_handler(
    mut scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    let delta = &mut scroll.delta;
    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.x > 0. {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.
        };

        if !max {
            scroll_position.x += delta.x;
            // Consume the X portion of the scroll delta.
            delta.x = 0.;
        }
    }

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.y > 0. {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.
        };

        if !max {
            scroll_position.y += delta.y;
            // Consume the Y portion of the scroll delta.
            delta.y = 0.;
        }
    }

    // Stop propagating when the delta is fully consumed.
    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
}
