use clap::{command, Parser};
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

mod tui;
mod widgets;

pub(crate) type ProjId = u32;

#[derive(Debug)]
struct App {
    exit: bool,
    main_project_list: ProjectList,
    rx: Receiver<TableEntry>,
    proj_count: u32,
    state: RuntimeState,
    show_empty: bool,
}

impl App {
    fn new(rx: Receiver<TableEntry>) -> Self {
        Self {
            exit: false,
            rx,
            main_project_list: ProjectList::default(),
            proj_count: 0,
            state: RuntimeState::MainListView,
            show_empty: false,
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

        frame.render_widget(&mut self.main_project_list, area);

        if matches!(self.state, RuntimeState::DisplayHelp) {
            self.render_help(frame);
        }

        if let RuntimeState::Selected(selected_proj) = &mut self.state {
            // TODO: something better than linear scan?
            if let Some(table_entry) = self
                .main_project_list
                .items
                .iter()
                .find(|i| i.id == selected_proj.id)
            {
                let result = selected_proj.render(frame, table_entry);
                if matches!(result, widgets::selected::SelectedWidgetResult::Finished) {
                    self.state = RuntimeState::MainListView;
                }
            } else {
                self.state = RuntimeState::MainListView;
            }
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let mut new_table_entry = false;
        while let Ok(res) = self.rx.try_recv() {
            self.main_project_list.biggest_artifact_bytes = self
                .main_project_list
                .biggest_artifact_bytes
                .max(res.artifact_bytes);
            if let Some((last_modified, _)) = res.last_modified_secs {
                self.main_project_list.oldest_modified_seconds = self
                    .main_project_list
                    .oldest_modified_seconds
                    .max(last_modified);
            }
            self.main_project_list.items.push(res);
            self.proj_count += 1;
            new_table_entry = true;
        }
        if new_table_entry {
            self.main_project_list
                .items
                .sort_unstable_by(|a, b| b.artifact_bytes.cmp(&a.artifact_bytes));
        }

        if !event::poll(EVENT_POLL_DURATION)? {
            return Ok(());
        }

        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.state {
            RuntimeState::MainListView => {
                match self.main_project_list.handle_key_event(key_event) {
                    widgets::project_list::ProjectListHandleKeyOutcome::Quit => todo!(),
                    widgets::project_list::ProjectListHandleKeyOutcome::Unused => todo!(),
                }
            }
            RuntimeState::DisplayHelp => todo!(),
            RuntimeState::Selected(_) => todo!(),
        }
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => match self.state {
                RuntimeState::MainListView => self.exit(),
                _ => self.state = RuntimeState::MainListView,
            },
            // KeyCode::Left | KeyCode::Char('h') => self.decrement_counter(),
            // KeyCode::Right | KeyCode::Char('l') => self.increment_counter(),
            KeyCode::Down | KeyCode::Char('j') => self.main_project_list.key_down_arrow(),
            KeyCode::Up | KeyCode::Char('k') => self.main_project_list.key_up_arrow(),
            KeyCode::Char('?') => self.toggle_help(),
            KeyCode::Enter => {
                if let Some(selected_idx) = self.main_project_list.table_state.selected() {
                    if let Some(selected_item) = self.main_project_list.items.get(selected_idx) {
                        self.state = RuntimeState::Selected(SelectedProject::new(selected_item));
                    }
                }
            }
            _ => {}
        }
    }

    fn toggle_help(&mut self) {
        self.state = match self.state {
            RuntimeState::DisplayHelp => RuntimeState::MainListView,
            _ => RuntimeState::DisplayHelp,
        };
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn render_help(&self, frame: &mut Frame) {
        let block = Block::default().title("Popup").borders(Borders::ALL);
        let area = centered_rect(60, 20, frame.area());
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(block, area);
    }

    fn render_selected(&self, proj_id: ProjId, frame: &mut Frame) {
        let Some(selected) = self
            .main_project_list
            .items
            .iter()
            .find(|proj| proj.id == proj_id)
        else {
            return;
        };

        let area = frame.area();

        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: (area.height / 3).max(4),
        };

        let selected_path = Path::new(selected.path_str.as_ref());

        let root_artifacts = selected.proj.root_artifacts(selected_path);

        let para = root_artifacts
            .into_iter()
            .map(|pb| {
                pb.strip_prefix(selected_path)
                    .unwrap_or(&pb)
                    .to_string_lossy()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");

        let bad_popup = ratatui::widgets::Paragraph::new(para)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .style(Style::new().yellow())
            .block(
                Block::new()
                    .title(selected.name.as_ref())
                    .title_style(Style::new().white().bold())
                    .borders(Borders::ALL)
                    .border_style(Style::new().red()),
            );
        frame.render_widget(Clear, popup_area);
        frame.render_widget(bad_popup, popup_area);
    }
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
            };

            if ttx.send(entry).is_err() {
                break;
            }
        }
    });

    let mut app = App::new(rrx);

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
