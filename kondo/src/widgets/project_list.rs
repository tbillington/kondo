use std::path::PathBuf;

use kondo_lib::{crossbeam::Receiver, Project as _, ProjectEnum};
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent},
    palette::Hsl,
    prelude::*,
    widgets::{block::Title, Cell, Row, Table, TableState},
};

use crate::{
    component::{Action, Component},
    discovery::discover,
    TableEntry,
};

use super::selected::SelectedProject;

pub(crate) enum ProjectListHandleKeyOutcome {
    Quit,
    Unused,
    Consumed,
    Select(SelectedProject),
}

#[derive(Debug)]
pub(crate) struct ProjectList {
    pub(crate) items: Vec<TableEntry>,
    pub(crate) item_rx: Receiver<TableEntry>,
    pub(crate) proj_count: u32,
    pub(crate) table_state: TableState,
    pub(crate) biggest_artifact_bytes: u64,
    pub(crate) oldest_modified_seconds: u64,
}

impl ProjectList {
    pub(crate) fn new(paths: Vec<PathBuf>) -> Self {
        Self {
            item_rx: discover(paths),
            items: Default::default(),
            proj_count: Default::default(),
            table_state: Default::default(),
            biggest_artifact_bytes: Default::default(),
            oldest_modified_seconds: Default::default(),
        }
    }
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

    pub(crate) fn toggle_stage_selected(&mut self) {
        if let Some(idx) = self.table_state.selected() {
            if let Some(item) = self.items.get_mut(idx) {
                item.staged = !item.staged;
            }
        }
    }

    // pub(crate) fn handle_key_event(&mut self, key_event: KeyEvent) -> ProjectListHandleKeyOutcome {
    //     match key_event.code {
    //         KeyCode::Char('q') | KeyCode::Esc => ProjectListHandleKeyOutcome::Quit,
    //         KeyCode::Down | KeyCode::Char('j') => {
    //             self.key_down_arrow();
    //             ProjectListHandleKeyOutcome::Consumed
    //         }
    //         KeyCode::Up | KeyCode::Char('k') => {
    //             self.key_up_arrow();
    //             ProjectListHandleKeyOutcome::Consumed
    //         }
    //         KeyCode::Enter => {
    //             if let Some(selected_idx) = self.table_state.selected() {
    //                 if let Some(selected_item) = self.items.get(selected_idx) {
    //                     return ProjectListHandleKeyOutcome::Select(SelectedProject::new(
    //                         selected_item,
    //                     ));
    //                 }
    //             }
    //             // log error?
    //             ProjectListHandleKeyOutcome::Consumed
    //         }
    //         _ => ProjectListHandleKeyOutcome::Unused,
    //     }
    // }
}

impl Component for ProjectList {
    fn handle_events(
        &mut self,
        event: Option<ratatui::crossterm::event::Event>,
    ) -> crate::component::Action {
        match event {
            // Some(Event::Quit) => Action::Quit,
            // Some(Event::Tick) => Action::Tick,
            Some(Event::Key(key_event)) => self.handle_key_events(key_event),
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event),
            // Some(Event::Resize(x, y)) => Action::Resize(x, y),
            Some(_) => Action::Noop,
            None => Action::Noop,
        }
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Down | KeyCode::Char('j') => {
                self.key_down_arrow();
                Action::Consumed
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.key_up_arrow();
                Action::Consumed
            }
            KeyCode::Char(' ') => {
                self.toggle_stage_selected();
                Action::Consumed
            }
            KeyCode::Enter => {
                if let Some(selected_idx) = self.table_state.selected() {
                    if let Some(selected_item) = self.items.get(selected_idx) {
                        return Action::Push(Box::new(SelectedProject::new(selected_item)));
                    }
                }
                // log error?
                Action::Consumed
            }
            _ => Action::Noop,
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        Widget::render(self, area, buf);
    }

    fn status_line(&self) -> Title {
        Title::from(
            Line::from(vec![
                "[".into(),
                "c".bold(),
                "]".into(),
                "lean".into(),
                " ".into(),
                "↑↓←→".into(),
                " ".into(),
                "[".into(),
                "h?".bold(),
                "]".into(),
                "elp".bold(),
            ])
            .yellow(),
        )
    }
}

impl Widget for &mut ProjectList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // eww why is this in render. in handle_events means it doesn't get called if there isn't a keypress etc
        let mut new_table_entry = false;
        while let Ok(res) = self.item_rx.try_recv() {
            self.biggest_artifact_bytes = self.biggest_artifact_bytes.max(res.artifact_bytes);
            if let Some((last_modified, _)) = res.last_modified_secs {
                self.oldest_modified_seconds = self.oldest_modified_seconds.max(last_modified);
            }
            self.items.push(res);
            self.proj_count += 1;
            new_table_entry = true;
        }
        if new_table_entry {
            self.items
                .sort_unstable_by(|a, b| b.artifact_bytes.cmp(&a.artifact_bytes));
        }

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
        let column_spacing = 2;

        // let rects = Layout::horizontal(&columns)
        //     .spacing(column_spacing)
        //     .split(area);

        // let path_column_width = rects[1].width as usize;

        // TODO: only render the visible rows
        let rows = self.items.iter().enumerate().map(|(row_index, proj)| {
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

            let name_with_staged = if proj.staged {
                format!("STAGED {}", proj.name)
            } else {
                proj.name.to_string()
            };

            let name = match &proj.focus {
                None => Text::from(name_with_staged),
                Some(focus) => Text::from(Line::default().spans([
                    Span::raw(name_with_staged),
                    Span::raw(" "),
                    Span::raw(focus.as_ref()).style(Color::from_hsl(Hsl::new(0.0, 0.0, 50.0))),
                ])),
            };

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
                    .style(Color::from_hsl(Hsl::new(
                        190.0,
                        last_modified_saturation as f32,
                        60.0,
                    )))
                    .alignment(Alignment::Right)
            } else {
                Text::raw("")
            };
            let size = Text::from(Line::default().spans([
                Span::raw(proj.artifact_bytes_fmt.0.as_ref()).style(Color::from_hsl(Hsl::new(
                    100.0,
                    artifact_size_saturation as f32,
                    50.0,
                ))),
                Span::raw(" "),
                Span::raw(proj.artifact_bytes_fmt.1.as_ref()).style(Color::from_hsl(Hsl::new(
                    100.0,
                    artifact_size_saturation as f32 - 20.0,
                    50.0,
                ))),
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
            // .block(block)
            .row_highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray),
            );

        ratatui::widgets::StatefulWidget::render(table, area, buf, &mut self.table_state);
    }
}
