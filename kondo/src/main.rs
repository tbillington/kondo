use clap::{command, Parser};
use component::Component;
use core::str;
use kondo_lib::{crossbeam::Receiver, Project, ProjectEnum};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, Borders, Cell, Clear, Row, Table, TableState,
    },
};
use std::{
    io,
    path::{Path, PathBuf},
    time::Duration,
};
use widgets::{project_list::ProjectList, selected::SelectedProject};

mod component;
mod discovery;
mod tui;
mod widgets;

pub(crate) type ProjId = u32;

struct App {
    exit: bool,
    // main_project_list: ProjectList,
    // rx: Receiver<TableEntry>,
    proj_count: u32,
    state: RuntimeState,
    show_empty: bool,
    component_stack: Vec<Box<dyn Component>>,
}

impl App {
    fn new(paths: Vec<PathBuf>) -> Self {
        Self {
            exit: false,
            // rx,
            // main_project_list: ProjectList::default(),
            proj_count: 0,
            state: RuntimeState::MainListView,
            show_empty: false,
            component_stack: vec![Box::new(ProjectList::new(paths))],
        }
    }
}

#[derive(Debug)]
enum RuntimeState {
    MainListView,
    DisplayHelp,
    Selected(SelectedProject),
}

const EVENT_POLL_DURATION: Duration = Duration::from_millis(16);

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let area_without_status_bar = {
            let mut area = area;
            area.height = area.height.saturating_sub(1);
            area
        };

        self.component_stack.iter_mut().for_each(|x| {
            x.render(area_without_status_bar, frame.buffer_mut());
        });

        let status_bar_area = {
            let mut area = area;
            area.y = area.height.saturating_sub(1);
            area.height = 1;
            area
        };

        if let Some(top_cmp) = self.component_stack.last() {
            Block::new()
                .title(top_cmp.status_line().alignment(Alignment::Center))
                .render(status_bar_area, frame.buffer_mut());
        }

        // frame.render_widget(&mut self.main_project_list, area);

        // if matches!(self.state, RuntimeState::DisplayHelp) {
        //     self.render_help(frame);
        // }

        // if let RuntimeState::Selected(selected_proj) = &mut self.state {
        //     // TODO: something better than linear scan?
        //     if let Some(table_entry) = self
        //         .main_project_list
        //         .items
        //         .iter()
        //         .find(|i| i.id == selected_proj.id)
        //     {
        //         let result = selected_proj.render(frame, table_entry);
        //         if matches!(result, widgets::selected::SelectedWidgetResult::Finished) {
        //             self.state = RuntimeState::MainListView;
        //         }
        //     } else {
        //         self.state = RuntimeState::MainListView;
        //     }
        // }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // let mut new_table_entry = false;
        // while let Ok(res) = self.rx.try_recv() {
        //     self.main_project_list.biggest_artifact_bytes = self
        //         .main_project_list
        //         .biggest_artifact_bytes
        //         .max(res.artifact_bytes);
        //     if let Some((last_modified, _)) = res.last_modified_secs {
        //         self.main_project_list.oldest_modified_seconds = self
        //             .main_project_list
        //             .oldest_modified_seconds
        //             .max(last_modified);
        //     }
        //     self.main_project_list.items.push(res);
        //     self.proj_count += 1;
        //     new_table_entry = true;
        // }
        // if new_table_entry {
        //     self.main_project_list
        //         .items
        //         .sort_unstable_by(|a, b| b.artifact_bytes.cmp(&a.artifact_bytes));
        // }

        if !event::poll(EVENT_POLL_DURATION)? {
            return Ok(());
        }

        let event = event::read()?;

        let mut remove_components_from_idx = None;
        let mut push_component = None;

        for (i, c) in self.component_stack.iter_mut().enumerate().rev() {
            match c.handle_events(Some(event.clone())) {
                component::Action::Quit => {
                    remove_components_from_idx = Some(i);
                    break;
                }
                component::Action::Consumed => {
                    break;
                }
                component::Action::Noop => {}
                component::Action::Push(c) => {
                    push_component = Some(c);
                    break;
                }
            }
        }

        if let Some(i) = remove_components_from_idx {
            self.component_stack.truncate(i);
        }

        if let Some(c) = push_component {
            self.component_stack.push(c);
        }

        if self.component_stack.is_empty() {
            self.exit = true;
        }

        // match event::read()? {
        //     // it's important to check that the event is a key press event as
        //     // crossterm also emits key release and repeat events on Windows.
        //     Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
        //         self.handle_key_event(key_event)
        //     }
        //     _ => {}
        // };

        Ok(())
    }

    // fn handle_key_event(&mut self, key_event: KeyEvent) {
    //     match self.state {
    //         RuntimeState::MainListView => {
    //             match self.main_project_list.handle_key_event(key_event) {
    //                 widgets::project_list::ProjectListHandleKeyOutcome::Quit => self.exit(),
    //                 widgets::project_list::ProjectListHandleKeyOutcome::Unused => {}
    //                 widgets::project_list::ProjectListHandleKeyOutcome::Consumed => {}
    //                 widgets::project_list::ProjectListHandleKeyOutcome::Select(selected) => {
    //                     self.state = RuntimeState::Selected(selected)
    //                 }
    //             }
    //         }
    //         RuntimeState::DisplayHelp => todo!(),
    //         RuntimeState::Selected(ref mut sp) => match sp.handle_key_event(key_event) {
    //             widgets::selected::SelectedProjectHandleKeyOutcome::Quit => {
    //                 self.state = RuntimeState::MainListView
    //             }
    //             widgets::selected::SelectedProjectHandleKeyOutcome::Unused => {}
    //         },
    //     }
    //     if let KeyCode::Char('?' | 'h') = key_event.code {
    //         self.toggle_help()
    //     };
    //     return;
    //     match key_event.code {
    //         KeyCode::Char('q') | KeyCode::Esc => match self.state {
    //             RuntimeState::MainListView => self.exit(),
    //             _ => self.state = RuntimeState::MainListView,
    //         },
    //         // KeyCode::Left | KeyCode::Char('h') => self.decrement_counter(),
    //         // KeyCode::Right | KeyCode::Char('l') => self.increment_counter(),
    //         KeyCode::Down | KeyCode::Char('j') => self.main_project_list.key_down_arrow(),
    //         KeyCode::Up | KeyCode::Char('k') => self.main_project_list.key_up_arrow(),
    //         KeyCode::Char('?') => self.toggle_help(),
    //         KeyCode::Enter => {
    //             if let Some(selected_idx) = self.main_project_list.table_state.selected() {
    //                 if let Some(selected_item) = self.main_project_list.items.get(selected_idx) {
    //                     self.state = RuntimeState::Selected(SelectedProject::new(selected_item));
    //                 }
    //             }
    //         }
    //         _ => {}
    //     }
    // }

    // fn toggle_help(&mut self) {
    //     self.state = match self.state {
    //         RuntimeState::DisplayHelp => RuntimeState::MainListView,
    //         _ => RuntimeState::DisplayHelp,
    //     };
    // }

    // fn exit(&mut self) {
    //     self.exit = true;
    // }

    fn render_help(&self, frame: &mut Frame) {
        let block = Block::default().title("Popup").borders(Borders::ALL);
        let area = centered_rect(60, 20, frame.area());
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(block, area);
    }

    // fn render_selected(&self, proj_id: ProjId, frame: &mut Frame) {
    //     let Some(selected) = self
    //         .main_project_list
    //         .items
    //         .iter()
    //         .find(|proj| proj.id == proj_id)
    //     else {
    //         return;
    //     };

    //     let area = frame.area();

    //     let popup_area = Rect {
    //         x: area.width / 4,
    //         y: area.height / 3,
    //         width: area.width / 2,
    //         height: (area.height / 3).max(4),
    //     };

    //     let selected_path = Path::new(selected.path_str.as_ref());

    //     let root_artifacts = selected.proj.root_artifacts(selected_path);

    //     let para = root_artifacts
    //         .into_iter()
    //         .map(|pb| {
    //             pb.strip_prefix(selected_path)
    //                 .unwrap_or(&pb)
    //                 .to_string_lossy()
    //                 .to_string()
    //         })
    //         .collect::<Vec<_>>()
    //         .join("\n");

    //     let bad_popup = ratatui::widgets::Paragraph::new(para)
    //         .wrap(ratatui::widgets::Wrap { trim: true })
    //         .style(Style::new().yellow())
    //         .block(
    //             Block::new()
    //                 .title(selected.name.as_ref())
    //                 .title_style(Style::new().white().bold())
    //                 .borders(Borders::ALL)
    //                 .border_style(Style::new().red()),
    //         );
    //     frame.render_widget(Clear, popup_area);
    //     frame.render_widget(bad_popup, popup_area);
    // }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

// impl Widget for &App {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let title = Title::from(" Counter App Tutorial ".bold());
//         let instructions = Title::from(Line::from(vec![
//             " Decrement ".into(),
//             "<Left>".blue().bold(),
//             " Increment ".into(),
//             "<Right>".blue().bold(),
//             " Quit ".into(),
//             "<Q> ".blue().bold(),
//         ]));
//         let block = Block::default()
//             .title(title.alignment(Alignment::Center))
//             .title(
//                 instructions
//                     .alignment(Alignment::Center)
//                     .position(Position::Bottom),
//             )
//             .borders(Borders::ALL)
//             .border_set(border::THICK);

//         let counter_text = Text::from(vec![Line::from(vec![
//             "Value: ".into(),
//             self.counter.to_string().yellow(),
//         ])]);

//         let items = ["Item 1", "Item 2", "Item 3"];
//         ratatui::widgets::Widget::render(
//             List::new(items)
//                 .block(block)
//                 .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
//                 .highlight_symbol(">>")
//                 .repeat_highlight_symbol(true),
//             area,
//             buf,
//         );

//         return;

//         Paragraph::new(counter_text)
//             .centered()
//             .block(block)
//             .render(area, buf);
//     }
// }

#[derive(Debug, Clone)]
struct TableEntry {
    id: ProjId,
    proj: ProjectEnum,
    name: Box<str>,
    focus: Option<Box<str>>,
    path: PathBuf,
    path_str: Box<str>,
    path_chars: u16,
    artifact_bytes: u64,
    artifact_bytes_fmt: (Box<str>, Box<str>),
    last_modified_secs: Option<(u64, Box<str>)>,
}

#[derive(Parser, Debug)]
#[command(name = "kondo")]
struct Opt {
    /// The directories to examine. Current working directory will be used if DIRS is omitted.
    #[arg(name = "DIRS")]
    dirs: Vec<PathBuf>,
}

fn main() -> io::Result<()> {
    let opt = Opt::parse();

    let mut terminal = tui::init()?;

    let dirs = if !opt.dirs.is_empty() {
        opt.dirs
    } else {
        vec![std::env::current_dir().unwrap()]
    };

    let mut app = App::new(dirs);

    let app_result = app.run(&mut terminal);
    tui::restore()?;

    println!("Proj Count: {}", app.proj_count);

    app_result
}

fn pretty_size(size: u64) -> String {
    const KIBIBYTE: u64 = 1024;
    const MEBIBYTE: u64 = 1_048_576;
    const GIBIBYTE: u64 = 1_073_741_824;
    const TEBIBYTE: u64 = 1_099_511_627_776;
    const PEBIBYTE: u64 = 1_125_899_906_842_624;
    const EXBIBYTE: u64 = 1_152_921_504_606_846_976;

    let (size, symbol) = match size {
        size if size < KIBIBYTE => (size as f64, "B"),
        size if size < MEBIBYTE => (size as f64 / KIBIBYTE as f64, "KiB"),
        size if size < GIBIBYTE => (size as f64 / MEBIBYTE as f64, "MiB"),
        size if size < TEBIBYTE => (size as f64 / GIBIBYTE as f64, "GiB"),
        size if size < PEBIBYTE => (size as f64 / TEBIBYTE as f64, "TiB"),
        size if size < EXBIBYTE => (size as f64 / PEBIBYTE as f64, "PiB"),
        _ => (size as f64 / EXBIBYTE as f64, "EiB"),
    };

    format!("{:.1}{}", size, symbol)
}

pub(crate) fn pretty_size2(size: u64) -> (Box<str>, Box<str>) {
    const KIBIBYTE: u64 = 1024;
    const MEBIBYTE: u64 = 1_048_576;
    const GIBIBYTE: u64 = 1_073_741_824;
    const TEBIBYTE: u64 = 1_099_511_627_776;
    const PEBIBYTE: u64 = 1_125_899_906_842_624;
    const EXBIBYTE: u64 = 1_152_921_504_606_846_976;

    let (size, symbol) = match size {
        size if size < KIBIBYTE => (size as f64, "B"),
        size if size < MEBIBYTE => (size as f64 / KIBIBYTE as f64, "KiB"),
        size if size < GIBIBYTE => (size as f64 / MEBIBYTE as f64, "MiB"),
        size if size < TEBIBYTE => (size as f64 / GIBIBYTE as f64, "GiB"),
        size if size < PEBIBYTE => (size as f64 / TEBIBYTE as f64, "TiB"),
        size if size < EXBIBYTE => (size as f64 / PEBIBYTE as f64, "PiB"),
        _ => (size as f64 / EXBIBYTE as f64, "EiB"),
    };

    let precision = if size < 10.0 { 1 } else { 0 };

    (
        format!("{:.*}", precision, size).into_boxed_str(),
        symbol.to_owned().into_boxed_str(),
    )
}

pub fn print_elapsed(secs: u64) -> Box<str> {
    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;
    // const WEEK: u64 = DAY * 7;
    // const MONTH: u64 = WEEK * 4;
    // const YEAR: u64 = DAY * 365;

    // let (unit, fstring) = match secs {
    //     secs if secs < MINUTE => (secs as f64, "second"),
    //     secs if secs < HOUR * 2 => (secs as f64 / MINUTE as f64, "minute"),
    //     secs if secs < DAY * 2 => (secs as f64 / HOUR as f64, "hour"),
    //     secs if secs < WEEK * 2 => (secs as f64 / DAY as f64, "day"),
    //     secs if secs < MONTH * 2 => (secs as f64 / WEEK as f64, "week"),
    //     secs if secs < YEAR * 2 => (secs as f64 / MONTH as f64, "month"),
    //     secs => (secs as f64 / YEAR as f64, "year"),
    // };

    // let (unit, fstring) = match secs {
    //     secs if secs < MINUTE => (secs as f64, "s"),
    //     secs if secs < HOUR * 2 => (secs as f64 / MINUTE as f64, "m"),
    //     secs if secs < DAY * 2 => (secs as f64 / HOUR as f64, "h"),
    //     secs if secs < WEEK * 2 => (secs as f64 / DAY as f64, "d"),
    //     // secs if secs < MONTH * 2 => (secs as f64 / WEEK as f64, "w"),
    //     secs if secs < YEAR * 2 => (secs as f64 / MONTH as f64, "M"),
    //     secs => (secs as f64 / YEAR as f64, "y"),
    // };

    // let unit = unit.round();

    let days = (secs as f64 / DAY as f64).round() as u64;

    format!("{days}d").into_boxed_str()

    // let plural = if unit == 1.0 { "" } else { "s" };

    // format!("{unit:.0}{fstring}").into_boxed_str()
}
