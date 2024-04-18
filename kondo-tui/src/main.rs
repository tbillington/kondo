use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use kondo_lib::{crossbeam::Receiver, Project, ProjectEnum};
use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, Borders, Cell, Clear, Row, Table, TableState,
    },
};
use std::{io, path::PathBuf, time::Duration};

mod tui;

#[derive(Debug)]
pub struct App {
    exit: bool,
    the_list: ProjectList,
    rx: Receiver<TableEntry>,
    proj_count: u32,
    display_help: bool,
}

const EVENT_POLL_DURATION: Duration = Duration::from_millis(16);

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        let area = frame.size();
        frame.render_widget(&mut self.the_list, area);
        // frame.render_widget(self, frame.size());

        if self.display_help {
            let block = Block::default().title("Popup").borders(Borders::ALL);
            let area = centered_rect(60, 20, area);
            frame.render_widget(Clear, area); //this clears out the background
            frame.render_widget(block, area);
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
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let mut new_table_entry = false;
        while let Ok(res) = self.rx.try_recv() {
            self.the_list.biggest_artifact_bytes =
                self.the_list.biggest_artifact_bytes.max(res.artifact_bytes);
            self.the_list.items.push(res);
            self.proj_count += 1;
            new_table_entry = true;
        }
        if new_table_entry {
            self.the_list
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
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            // KeyCode::Left | KeyCode::Char('h') => self.decrement_counter(),
            // KeyCode::Right | KeyCode::Char('l') => self.increment_counter(),
            KeyCode::Down | KeyCode::Char('j') => self.the_list.key_down_arrow(),
            KeyCode::Up | KeyCode::Char('k') => self.the_list.key_up_arrow(),
            KeyCode::Char('?') => self.display_help(!self.display_help),
            _ => {}
        }
    }

    fn display_help(&mut self, show: bool) {
        self.display_help = show;
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

#[derive(Debug, Default)]
struct ProjectList {
    items: Vec<TableEntry>,
    // list_state: ListState,
    table_state: TableState,
    biggest_artifact_bytes: u64,
}

impl ProjectList {
    fn key_down_arrow(&mut self) {
        if self.items.is_empty() {
            self.table_state.select(None);
            return;
        }

        match self.table_state.selected() {
            Some(idx) => self
                .table_state
                .select(Some((idx + 1).min(self.items.len() - 1))),
            None => self.table_state.select(Some(0)),
        }
    }

    fn key_up_arrow(&mut self) {
        if self.items.is_empty() {
            self.table_state.select(None);
            return;
        }

        match self.table_state.selected() {
            Some(idx) => self.table_state.select(Some(idx.saturating_sub(1))),
            None => self.table_state.select(Some(0)),
        }
    }
}

impl Widget for &mut ProjectList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // let title = Title::from(" Kondo 🧹 ".bold());
        let instructions = Title::from(
            Line::from(vec![
                "[".into(),
                "?".bold(),
                "]".into(),
                "elp".bold(),
                " ".into(),
                // "<Q> ".blue().bold(),
            ])
            .yellow(),
        );
        let block = Block::default()
        //     .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
        //     // .borders(Borders::TOP.union(Borders::BOTTOM))
        //     // .border_set(border::ROUNDED)
            ;

        // let counter_text = Text::from(vec![Line::from(vec![
        //     "Value: ".into(),
        //     self.counter.to_string().yellow(),
        // ])]);

        // let items = ["Item 1", "Item 2", "Item 3"];

        let columns = Constraint::from_fills([2, 5, 1, 1, 1]);
        let column_spacing = 2;

        let rects = Layout::horizontal(&columns)
            // .constraints(&)
            .spacing(column_spacing)
            .split(area);

        let path_column_width = rects[1].width as usize;

        let rows = self.items.iter().map(|proj| {
            // let name = Text::from(proj.1.name(&proj.0).unwrap_or_default());

            // let mut path = proj.0.to_string_lossy().into_owned();

            // let char_count = path.chars().count();

            // if char_count > path_column_width {
            //     path = path
            //         .chars()
            //         .skip(char_count - path_column_width)
            //         .take(path_column_width)
            //         .collect();
            // }

            fn proj_colour(proj: ProjectEnum) -> Color {
                // https://github.com/ozh/github-colors/blob/master/colors.json
                match proj {
                    ProjectEnum::RustProject(_) => Color::from_u32(0xdea584),
                    ProjectEnum::NodeProject(_) => Color::from_u32(0xf1e05a),
                    ProjectEnum::UnityProject(_) => Color::from_u32(0x178600),
                }
            }

            fn lerp(start: f64, end: f64, t: f64) -> f64 {
                ((1.0 - t) * start) + (t * end)
            }
            fn inv_lerp(start: f64, end: f64, t: f64) -> f64 {
                (t - start) / (end - start)
            }
            fn remap(src_start: f64, src_end: f64, dest_start: f64, dest_end: f64, t: f64) -> f64 {
                let rel = inv_lerp(src_start, src_end, t);
                lerp(dest_start, dest_end, rel)
            }

            let t = (proj.artifact_bytes as f64).sqrt();
            let rel = inv_lerp(0.0, (self.biggest_artifact_bytes as f64).sqrt(), t);

            let saturation = lerp(20.0, 100.0, rel);

            // let file_size_greenness = remap(
            //     0.0,
            //     (self.biggest_artifact_bytes as f64).sqrt(),
            //     0.2,
            //     100.0,
            //     (proj.artifact_bytes as f64).sqrt(),
            // );

            // let path = Text::from(path).dark_gray();
            // let kind = Text::from(proj.1.kind_name()).style(proj_colour(proj.1));

            let name = Text::from(proj.name.as_ref());
            let path = Text::from(proj.path.as_ref()).dark_gray();

            let last_mod = if let Some(lm) = &proj.last_modified_secs {
                Text::from(lm.1.as_ref())
                    .light_blue()
                    .alignment(Alignment::Right)
            } else {
                Text::raw("")
            };
            //  Text::from(
            //     proj.last_modified_secs
            //         .map(|lm| lm.1.as_ref())
            //         .unwrap_or(""),
            // );
            let size = Text::from(
                Line::default().spans([
                    Span::raw(proj.artifact_bytes_fmt.0.as_ref())
                        .bold()
                        .style(Color::from_hsl(100.0, saturation, 50.0)),
                    Span::raw(" "),
                    Span::raw(proj.artifact_bytes_fmt.1.as_ref()).style(Color::from_hsl(
                        100.0,
                        saturation - 20.0,
                        50.0,
                    )),
                ]),
            )
            .alignment(Alignment::Right);
            let kind = Text::from(proj.proj.kind_name()).style(proj_colour(proj.proj));

            Row::new(vec![name, path, last_mod, size, kind])
        });

        let table = Table::new(rows, columns)
            .header(
                Row::new(vec![
                    Cell::new("Project"),
                    Cell::new("Path"),
                    Cell::new("Mod"),
                    Cell::new("Size"),
                    Cell::new("Type"),
                ])
                .underlined()
                .light_blue()
                .bold(),
            )
            .column_spacing(column_spacing)
            .block(block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray),
            );

        ratatui::widgets::StatefulWidget::render(table, area, buf, &mut self.table_state);

        // let l = self
        //     .items
        //     .iter()
        //     .map(|i| Text::from(i.1.name(&i.0).unwrap_or_else(|| "Unknown".to_string())))
        //     .collect::<List>()
        //     .block(block)
        //     .highlight_style(
        //         Style::default()
        //             .add_modifier(Modifier::BOLD)
        //             .bg(Color::DarkGray),
        //     )
        //     // .highlight_symbol(">")
        //     .repeat_highlight_symbol(true);

        // ratatui::widgets::StatefulWidget::render(&l, area, buf, &mut self.list_state);
        // area,
        // buf,
    }
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

#[derive(Debug)]
struct TableEntry {
    proj: ProjectEnum,
    name: Box<str>,
    path: Box<str>,
    path_chars: u16,
    artifact_bytes: u64,
    artifact_bytes_fmt: (Box<str>, Box<str>),
    last_modified_secs: Option<(u64, Box<str>)>,
}

fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;

    let rx = kondo_lib::run_local([PathBuf::from("/Users/choc/code")].into_iter(), None);
    let (ttx, rrx) = kondo_lib::crossbeam::unbounded();
    std::thread::spawn(move || {
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

            let artifact_bytes = proj.artifact_size(&path);

            if artifact_bytes == 0 {
                continue;
            }

            let artifact_bytes_fmt = pretty_size2(artifact_bytes);

            let mut last_modified_secs = None;
            if let Ok(lm) = proj.last_modified(&path) {
                if let Ok(elapsed) = lm.elapsed() {
                    let secs = elapsed.as_secs();
                    last_modified_secs = Some((secs, print_elapsed(secs)));
                }
            }

            let path = path.to_string_lossy().into_owned().into_boxed_str();

            let path_chars = path.chars().count() as u16;

            let entry = TableEntry {
                proj,
                name,
                path,
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

    let mut app = App {
        exit: false,
        rx: rrx,
        the_list: ProjectList::default(),
        proj_count: 0,
        display_help: false,
    };

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

fn pretty_size2(size: u64) -> (Box<str>, Box<str>) {
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
