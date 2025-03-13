use std::{error::Error, time::Duration};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{Block, Borders, HighlightSpacing, List, ListState, Padding, StatefulWidget, Widget},
    DefaultTerminal,
};

use crate::archive::Archive;

pub struct DisambiguationPrompt<'a> {
    archive_groups: Vec<Vec<&'a Archive>>,
    group_list_states: Vec<ListState>,
    longest_base_name: usize,
}

impl<'a> DisambiguationPrompt<'a> {
    pub fn new(archive_groups: Vec<Vec<&'a Archive>>) -> Self {
        let mut group_list_states: Vec<_> = archive_groups
            .iter()
            .map(|_| ListState::default())
            .collect();
        if !group_list_states.is_empty() {
            group_list_states[0].select_first();
        }

        let longest_base_name = archive_groups
            .iter()
            .map(|grp| {
                grp.iter()
                    .map(|arc| arc.base_name().len())
                    .max()
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0);

        DisambiguationPrompt {
            archive_groups,
            group_list_states,
            longest_base_name,
        }
    }

    pub fn run(self) -> Result<Option<Archive>, Box<dyn Error>> {
        let terminal = ratatui::init();
        let app_result = self.run_prompt_loop(terminal);
        ratatui::restore();
        app_result
    }

    fn run_prompt_loop(
        mut self,
        mut terminal: DefaultTerminal,
    ) -> Result<Option<Archive>, Box<dyn Error>> {
        loop {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_input(&key)? {
                        break;
                    }
                }
            }
        }
        Ok(self.selected_archive())
    }

    fn selected_archive(&self) -> Option<Archive> {
        for idx in 0..self.group_list_states.len() {
            let lst = &self.group_list_states[idx];
            if let Some(offset) = lst.selected() {
                return Some(self.archive_groups[idx][offset].clone());
            }
        }

        None
    }

    fn select_none(&mut self) {
        for idx in 0..self.group_list_states.len() {
            let lst = &mut self.group_list_states[idx];
            lst.select(None);
        }
    }

    fn select_next(&mut self) {
        for idx in 0..self.group_list_states.len() {
            let lst = &mut self.group_list_states[idx];

            if let Some(offset) = lst.selected() {
                if offset == self.archive_groups[idx].len() - 1 {
                    if idx == self.archive_groups.len() - 1 {
                        // no-op
                    } else {
                        lst.select(None);
                        self.group_list_states[idx + 1].select_first();
                    }
                } else {
                    lst.select_next();
                }
            }
        }
    }

    fn select_prev(&mut self) {
        for idx in 0..self.group_list_states.len() {
            let lst = &mut self.group_list_states[idx];

            if let Some(offset) = lst.selected() {
                match offset {
                    0 => match idx {
                        0 => {}
                        _ => {
                            lst.select(None);
                            self.group_list_states[idx - 1].select_last();
                        }
                    },
                    _ => lst.select_previous(),
                }
            }
        }
    }

    fn handle_input(&mut self, key: &KeyEvent) -> Result<bool, Box<dyn Error>> {
        match key.code {
            KeyCode::Char('q') => {
                self.select_none();
                return Ok(true);
            }
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Enter => {
                return Ok(true);
            }
            _ => {}
        };

        Ok(false)
    }
}

impl<'a> Widget for &mut DisambiguationPrompt<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = &ratatui::layout::Layout::new(
            Direction::Vertical,
            self.archive_groups
                .iter()
                .map(|g| Constraint::Length(g.len().try_into().unwrap_or(1) + 4)),
        );
        let subareas = layout.split(area);

        for (idx, grp) in self.archive_groups.iter().enumerate() {
            let subarea = subareas[idx];

            let content_layout = Layout::new(
                Direction::Horizontal,
                vec![Constraint::Length(
                    (self.longest_base_name + 7)
                        .try_into()
                        .unwrap_or(area.width),
                )],
            );
            let content_areas = content_layout.split(subarea);

            let items: Vec<&str> = grp.iter().map(|arc| arc.base_name()).collect();

            let block = Block::new()
                .padding(Padding::uniform(1))
                .title(Line::raw(grp[0].name()))
                .borders(Borders::ALL);

            let list = List::new(items)
                .block(block)
                .highlight_symbol("* ")
                .highlight_spacing(HighlightSpacing::Always);

            StatefulWidget::render(
                list,
                content_areas[0],
                buf,
                &mut self.group_list_states[idx],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{
        buffer::Buffer,
        crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
        layout::Rect,
        widgets::Widget,
    };

    use crate::archive::Archive;

    use super::DisambiguationPrompt;

    fn construct_archive_groups() -> Vec<Vec<Archive>> {
        vec![
            vec![
                Archive::from_file_name("xyz-20240101.zip").unwrap(),
                Archive::from_file_name("xyz-20240201.zip").unwrap(),
                Archive::from_file_name("xyz-20240301.zip").unwrap(),
            ],
            vec![
                Archive::from_file_name("thing-abc-123-20240101.zip").unwrap(),
                Archive::from_file_name("thing-abc-123-20240102.zip").unwrap(),
            ],
        ]
    }

    #[test]
    fn test_basic_render() {
        let archive_groups = construct_archive_groups();
        let archive_group_refs = archive_groups
            .iter()
            .map(|grp| grp.iter().collect())
            .collect();
        let mut prompt = DisambiguationPrompt::new(archive_group_refs);

        let mut buf = Buffer::empty(Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 25,
        });
        prompt.render(buf.area, &mut buf);
        for (idx, expected_letter) in "xyz".chars().enumerate() {
            let observed_letter = buf.cell((1 + idx as u16, 0)).unwrap().symbol();
            assert_eq!(expected_letter, observed_letter.chars().next().unwrap());
        }

        assert_eq!("*", buf.cell((2, 2)).unwrap().symbol());
        assert_eq!(" ", buf.cell((2, 3)).unwrap().symbol());

        prompt
            .handle_input(&KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE))
            .expect("Prompt input failure.");
        prompt.render(buf.area, &mut buf);
        assert_eq!(" ", buf.cell((2, 2)).unwrap().symbol());
        assert_eq!("*", buf.cell((2, 3)).unwrap().symbol());

        prompt
            .handle_input(&KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE))
            .expect("Prompt input failure.");
        prompt.render(buf.area, &mut buf);
        assert_eq!("*", buf.cell((2, 2)).unwrap().symbol());
        assert_eq!(" ", buf.cell((2, 3)).unwrap().symbol());
    }
}
