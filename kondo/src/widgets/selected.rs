use std::sync::mpsc::{sync_channel, Receiver, TryRecvError};

use kondo_lib::{project::dir_size, Project as _};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, Paragraph},
};

use crate::{pretty_size2, ProjId, TableEntry};

#[derive(Debug)]
pub(crate) struct SelectedProject {
    pub(crate) id: ProjId,
    state: State,
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
        SelectedProject { id, state }
    }

    pub(crate) fn render(&mut self, frame: &mut Frame, proj: &TableEntry) -> SelectedWidgetResult {
        if let State::FetchingArtifactSizes(rx) = &self.state {
            match rx.try_recv() {
                Ok(artifact_info) => self.state = State::Cleanable(artifact_info),
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    eprintln!(
                        "expected artifact sizes but sender disconnected, proj: {}",
                        proj.path_str
                    );
                    return SelectedWidgetResult::Finished;
                }
            }
        }

        let selected = proj;
        let area = frame.area();

        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: (area.height / 3).max(4),
        };

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
                frame.render_widget(Clear, popup_area);
                frame.render_widget(popup, popup_area);
            }
            State::Cleanable(artifact_dirs) => {
                let artifact_list =
                    List::new(artifact_dirs.iter().map(|(dir, size_str, size_suffix)| {
                        Text::from(
                            Line::default().spans([
                                Span::raw(dir.as_ref()),
                                Span::raw(" "),
                                Span::raw(size_str.as_ref())
                                    .style(Color::from_hsl(100.0, 100.0, 50.0)),
                                Span::raw(" "),
                                Span::raw(size_suffix.as_ref())
                                    .style(Color::from_hsl(100.0, 80.0, 50.0)),
                            ]),
                        )
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
                frame.render_widget(Clear, popup_area);
                frame.render_widget(popup, popup_area);
            }
            State::Clean => todo!(),
        }

        SelectedWidgetResult::Okay
    }
}

// impl Widget for SelectedProject {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         todo!()
//     }
// }
