use crate::cache::{extract_quantization, format_size};
use crate::model::Model;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use std::io;

pub struct Ui {
    list_state: ListState,
    models: Vec<Model>,
    selected_index: usize,
}

impl Ui {
    pub fn new(models: Vec<Model>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            list_state,
            models,
            selected_index: 0,
        }
    }

    pub fn run(&mut self) -> Result<Option<Model>> {
        match enable_raw_mode() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to enable raw mode: {}", e);
                // Fallback to simple selection without UI
                return Ok(self.models.get(0).cloned());
            }
        }

        let mut stdout = io::stdout();
        match execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to execute terminal commands: {}", e);
                let _ = disable_raw_mode();
                // Fallback to simple selection without UI
                return Ok(self.models.get(0).cloned());
            }
        };

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = match Terminal::new(backend) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to create terminal: {}", e);
                let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
                let _ = disable_raw_mode();
                // Fallback to simple selection without UI
                return Ok(self.models.get(0).cloned());
            }
        };

        let result = self.run_terminal(&mut terminal);

        let _ = disable_raw_mode();
        let _ = execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = terminal.show_cursor();

        result
    }

    fn run_terminal(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<Option<Model>> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.move_up();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.move_down();
                        }
                        KeyCode::Enter | KeyCode::Char('o') => {
                            return Ok(self.models.get(self.selected_index).cloned());
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            return Ok(None);
                        }
                        KeyCode::PageUp => {
                            self.page_up();
                        }
                        KeyCode::PageDown => {
                            self.page_down();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(frame.area());

        let items: Vec<ListItem> = self
            .models
            .iter()
            .enumerate()
            .map(|(idx, model)| {
                let quant = extract_quantization(&model.name);
                let text = format!("{} - {} [{}]", model.name, format_size(model.size), quant);

                let style = if idx == self.selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(Span::styled(text, style)))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Models (↑↓ to navigate, Enter to select) "),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, chunks[0], &mut self.list_state);

        if let Some(selected_model) = self.models.get(self.selected_index) {
            let details_text = format!(
                "
Name: {}
Path: {}
Size: {}
Quantization: {}
MMProj: {},
",
                selected_model.name,
                selected_model.gguf_path.display(),
                format_size(selected_model.size),
                extract_quantization(&selected_model.name),
                selected_model
                    .mmproj_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "Not found".to_string())
            );

            let details = Paragraph::new(details_text)
                .block(Block::default().borders(Borders::ALL).title(" Details "))
                .wrap(Wrap { trim: false });

            frame.render_widget(details, chunks[1]);
        }
    }

    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    fn move_down(&mut self) {
        if self.selected_index + 1 < self.models.len() {
            self.selected_index += 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    fn page_up(&mut self) {
        let page_size = 5;
        if self.selected_index >= page_size {
            self.selected_index -= page_size;
        } else {
            self.selected_index = 0;
        }
        self.list_state.select(Some(self.selected_index));
    }

    fn page_down(&mut self) {
        let page_size = 5;
        let max_index = self.models.len().saturating_sub(1);
        if self.selected_index + page_size <= max_index {
            self.selected_index += page_size;
        } else {
            self.selected_index = max_index;
        }
        self.list_state.select(Some(self.selected_index));
    }
}
