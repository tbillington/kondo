use kondo_lib::{Project as _, ProjectEnum};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{block::Title, Block, Cell, Row, Table, TableState},
};

use crate::TableEntry;

use super::selected::SelectedProject;

pub(crate) enum ProjectListHandleKeyOutcome {
    Quit,
    Unused,
    Consumed,
    Select(SelectedProject),
}

#[derive(Debug, Default)]
pub(crate) struct ProjectList {
    pub(crate) items: Vec<TableEntry>,
    // list_state: ListState,
    pub(crate) table_state: TableState,
    pub(crate) biggest_artifact_bytes: u64,
    pub(crate) oldest_modified_seconds: u64,
}

impl ProjectList {
    pub(crate) fn key_down_arrow(&mut self) {
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

    pub(crate) fn key_up_arrow(&mut self) {
        if self.items.is_empty() {
            self.table_state.select(None);
            return;
        }

        match self.table_state.selected() {
            Some(idx) => self.table_state.select(Some(idx.saturating_sub(1))),
            None => self.table_state.select(Some(0)),
        }
    }

    pub(crate) fn handle_key_event(&mut self, key_event: KeyEvent) -> ProjectListHandleKeyOutcome {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => ProjectListHandleKeyOutcome::Quit,
            KeyCode::Down | KeyCode::Char('j') => {
                self.key_down_arrow();
                ProjectListHandleKeyOutcome::Consumed
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.key_up_arrow();
                ProjectListHandleKeyOutcome::Consumed
            }
            KeyCode::Enter => {
                if let Some(selected_idx) = self.table_state.selected() {
                    if let Some(selected_item) = self.items.get(selected_idx) {
                        return ProjectListHandleKeyOutcome::Select(SelectedProject::new(
                            selected_item,
                        ));
                    }
                }
                // log error?
                ProjectListHandleKeyOutcome::Consumed
            }
            _ => ProjectListHandleKeyOutcome::Unused,
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
                    .position(ratatui::widgets::block::Position::Bottom),
            )
        //     // .borders(Borders::TOP.union(Borders::BOTTOM))
        //     // .border_set(border::ROUNDED)
            ;

        // let counter_text = Text::from(vec![Line::from(vec![
        //     "Value: ".into(),
        //     self.counter.to_string().yellow(),
        // ])]);

        // let items = ["Item 1", "Item 2", "Item 3"];

        let columns = {
            let modified = Constraint::Length(3);

            let kind = Constraint::Length(
                self.items
                    .iter()
                    .map(|i| i.proj.kind_name().len())
                    .max()
                    .unwrap_or(4) as u16,
            );

            vec![
                Constraint::Fill(4),
                Constraint::Fill(3),
                modified,
                Constraint::Length(7),
                kind,
            ]
        };
        // std::fs::write("out.txt", format!("{:#?}", columns));
        let column_spacing = 2;

        let rects = Layout::horizontal(&columns)
            // .constraints(&)
            .spacing(column_spacing)
            .split(area);

        let path_column_width = rects[1].width as usize;

        // TODO: only render the visible rows
        let rows = self.items.iter().enumerate().map(|(row_index, proj)| {
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
                    ProjectEnum::CMakeProject(_) => Color::from_u32(0xda3434),
                    ProjectEnum::NodeProject(_) => Color::from_u32(0xf1e05a),
                    ProjectEnum::RustProject(_) => Color::from_u32(0xdea584),
                    ProjectEnum::UnityProject(_) => Color::from_u32(0x178600),
                    ProjectEnum::GodotProject(_) => Color::from_u32(0x355570),
                }
            }

            fn lerp(start: f64, end: f64, t: f64) -> f64 {
                ((1.0 - t) * start) + (t * end)
            }
            fn inv_lerp(start: f64, end: f64, t: f64) -> f64 {
                (t - start) / (end - start)
            }
            #[allow(unused)]
            fn remap(src_start: f64, src_end: f64, dest_start: f64, dest_end: f64, t: f64) -> f64 {
                let rel = inv_lerp(src_start, src_end, t);
                lerp(dest_start, dest_end, rel)
            }

            let artifact_size_saturation = {
                let t = (proj.artifact_bytes as f64).sqrt();
                let rel = inv_lerp(0.0, (self.biggest_artifact_bytes as f64).sqrt(), t);
                lerp(20.0, 100.0, rel)
            };

            let last_modified_saturation = {
                let t = match proj.last_modified_secs {
                    Some((m, _)) => m as f64,
                    None => 0.0,
                };
                let rel = inv_lerp(0.0, self.oldest_modified_seconds as f64, t);
                lerp(20.0, 100.0, rel)
            };

            // let file_size_greenness = remap(
            //     0.0,
            //     (self.biggest_artifact_bytes as f64).sqrt(),
            //     0.2,
            //     100.0,
            //     (proj.artifact_bytes as f64).sqrt(),
            // );

            // let path = Text::from(path).dark_gray();
            // let kind = Text::from(proj.1.kind_name()).style(proj_colour(proj.1));

            let name = match &proj.focus {
                None => Text::from(proj.name.as_ref()),
                Some(focus) => Text::from(Line::default().spans([
                    Span::raw(proj.name.as_ref()),
                    Span::raw(" "),
                    Span::raw(focus.as_ref()).style(Color::from_hsl(0.0, 0.0, 50.0)),
                ])),
            };

            // self.table_state.

            let mut path = Text::from(proj.path_str.as_ref()).dark_gray();

            if self
                .table_state
                .selected()
                .is_some_and(|selected_idx| selected_idx == row_index)
            {
                path = path.gray();
            }

            let last_mod = if let Some(lm) = &proj.last_modified_secs {
                Text::from(lm.1.as_ref())
                    .style(Color::from_hsl(190.0, last_modified_saturation, 60.0))
                    .alignment(Alignment::Right)
            } else {
                Text::raw("")
            };
            //  Text::from(
            //     proj.last_modified_secs
            //         .map(|lm| lm.1.as_ref())
            //         .unwrap_or(""),
            // );
            let size = Text::from(Line::default().spans([
                Span::raw(proj.artifact_bytes_fmt.0.as_ref()).style(Color::from_hsl(
                    100.0,
                    artifact_size_saturation,
                    50.0,
                )),
                Span::raw(" "),
                Span::raw(proj.artifact_bytes_fmt.1.as_ref()).style(Color::from_hsl(
                    100.0,
                    artifact_size_saturation - 20.0,
                    50.0,
                )),
            ]))
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
