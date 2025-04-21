use std::sync::mpsc::{sync_channel, Receiver, TryRecvError};

use kondo_lib::{project::dir_size, Project as _};
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent},
    palette::Hsl,
    prelude::*,
    widgets::{Block, Borders, Clear, List, Paragraph},
};

use crate::{
    component::{Action, Component},
    pretty_size2, ProjId, TableEntry,
};

#[derive(Debug)]
pub(crate) struct SelectedProject {
    pub(crate) id: ProjId,
    state: State,
    table_entry: TableEntry,
}

#[derive(Debug)]
enum State {
    FetchingArtifactSizes(Receiver<Vec<(Box<str>, Box<str>, Box<str>)>>),
    Cleanable(Vec<(Box<str>, Box<str>, Box<str>)>),
    Clean,
}

pub(crate) enum SelectedWidgetResult {
    Okay,
    Finished,
}

pub(crate) enum SelectedProjectHandleKeyOutcome {
    Quit,
    Unused,
}

impl SelectedProject {
    pub(crate) fn new(proj: &TableEntry) -> Self {
        let (tx, rx) = sync_channel(1);

        let proj_kind = proj.proj;
        let path = proj.path.clone();
        std::thread::spawn(move || {
            tx.send(
                proj_kind
                    .root_artifacts(&path)
                    .into_iter()
                    .map(|path| {
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string().into_boxed_str())
                            .unwrap_or_default();
                        let (size_str, size_suffix) = pretty_size2(dir_size(path));
                        (name, size_str, size_suffix)
                    })
                    .collect(),
            )
        });

        let id = proj.id;
        let state = State::FetchingArtifactSizes(rx);
        SelectedProject {
            id,
            state,
            table_entry: proj.clone(),
        }
    }

    pub(crate) fn render(&mut self, area: Rect, buf: &mut Buffer) -> SelectedWidgetResult {
        if let State::FetchingArtifactSizes(rx) = &self.state {
            match rx.try_recv() {
                Ok(artifact_info) => self.state = State::Cleanable(artifact_info),
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    eprintln!(
                        "expected artifact sizes but sender disconnected, proj: {}",
                        self.table_entry.path_str
                    );
                    return SelectedWidgetResult::Finished;
                }
            }
        }

        let selected = &self.table_entry;

        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: (area.height / 3).max(4),
        };

        let area = popup_area;

        match &self.state {
            State::FetchingArtifactSizes(_) => {
                let popup = Paragraph::new("Fetching")
                    .wrap(ratatui::widgets::Wrap { trim: true })
                    .style(Style::new().yellow())
                    .block(
                        Block::new()
                            .title(selected.name.as_ref())
                            .title_style(Style::new().white().bold())
                            .borders(Borders::ALL)
                            .border_style(Style::new().red()),
                    );
                Clear.render(area, buf);
                // buf.render_widget(Clear, popup_area);
                popup.render(area, buf);
                // frame.render_widget(popup, popup_area);
            }
            State::Cleanable(artifact_dirs) => {
                let colour_a = Color::from_hsl(Hsl::new(100.0, 100.0, 50.0));
                let colour_b = Color::from_hsl(Hsl::new(100.0, 80.0, 50.0));
                let artifact_list =
                    List::new(artifact_dirs.iter().map(|(dir, size_str, size_suffix)| {
                        Text::from(Line::default().spans([
                            Span::raw(dir.as_ref()),
                            Span::raw(" "),
                            Span::raw(size_str.as_ref()).style(colour_a),
                            Span::raw(" "),
                            Span::raw(size_suffix.as_ref()).style(colour_b),
                        ]))
                    }));
                // let para = artifact_dirs
                //     .into_iter()
                //     .map(|(dir, size)| format!("{dir} ({size})"))
                //     .collect::<Vec<_>>()
                //     .join("\n");

                let popup = //Paragraph::new(para)
                    artifact_list
                    // .wrap(ratatui::widgets::Wrap { trim: true })
                    .style(Style::new().yellow())
                    .block(
                        Block::new()
                            .title(selected.name.as_ref())
                            .title_style(Style::new().white().bold())
                            .borders(Borders::ALL)
                            .border_style(Style::new().red()),
                    );
                Clear.render(area, buf);
                Widget::render(popup, area, buf);
                // frame.render_widget(Clear, popup_area);
                // frame.render_widget(popup, popup_area);
            }
            State::Clean => todo!(),
        }

        SelectedWidgetResult::Okay
    }

    pub(crate) fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
    ) -> SelectedProjectHandleKeyOutcome {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => SelectedProjectHandleKeyOutcome::Quit,
            _ => SelectedProjectHandleKeyOutcome::Unused,
        }
    }
}

impl Component for SelectedProject {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        self.render(area, buf);
    }

    fn handle_events(&mut self, event: Option<Event>) -> crate::component::Action {
        match event {
            // Some(Event::Quit) => Action::Quit,
            // Some(Event::Tick) => Action::Tick,
            Some(Event::Key(key_event)) => self.handle_key_events(key_event),
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event),
            // Some(Event::Resize(x, y)) => Action::Resize(x, y),
            Some(_) => crate::component::Action::Noop,
            None => crate::component::Action::Noop,
        }
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> crate::component::Action {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Up
            | KeyCode::Char('k')
            | KeyCode::Down
            | KeyCode::Char('j')
            | KeyCode::Left
            | KeyCode::Char('h')
            | KeyCode::Right
            | KeyCode::Char('l') => Action::Consumed,
            _ => Action::Noop,
        }
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> crate::component::Action {
        crate::component::Action::Noop
    }

    fn update(&mut self, action: crate::component::Action) -> crate::component::Action {
        crate::component::Action::Noop
    }
}
