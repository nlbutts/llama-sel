use crate::cache::{add_llama_server, add_model_config, extract_quantization, format_size};
use crate::model::{GlobalConfig, LlamaServer, Model};
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

#[derive(PartialEq)]
enum SelectionStage {
    Server,
    Model,
    AddModel,
    AddServer,
}

pub struct Ui {
    list_state: ListState,
    models: Vec<Model>,
    servers: Vec<LlamaServer>,
    selected_index: usize,
    stage: SelectionStage,
    config: GlobalConfig,
    selected_model: Option<Model>,
}

impl Ui {
    pub fn new(models: Vec<Model>, config: GlobalConfig) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            list_state,
            models,
            servers: config.llama_servers.clone(),
            selected_index: 0,
            stage: SelectionStage::Server,
            config,
            selected_model: None,
        }
    }

    pub fn run(&mut self) -> Result<(Option<Model>, GlobalConfig)> {
        match enable_raw_mode() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to enable raw mode: {}", e);
                return Ok((self.models.get(0).cloned(), self.config.clone()));
            }
        }

        let mut stdout = io::stdout();
        match execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to execute terminal commands: {}", e);
                let _ = disable_raw_mode();
                return Ok((self.models.get(0).cloned(), self.config.clone()));
            }
        };

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = match Terminal::new(backend) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to create terminal: {}", e);
                let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
                let _ = disable_raw_mode();
                return Ok((self.models.get(0).cloned(), self.config.clone()));
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
    ) -> Result<(Option<Model>, GlobalConfig)> {
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
                            let should_continue = self.handle_enter();
                            if !should_continue {
                                return Ok((self.selected_model.take(), self.config.clone()));
                            }
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            return Ok((None, self.config.clone()));
                        }
                        KeyCode::PageUp => {
                            self.page_up();
                        }
                        KeyCode::PageDown => {
                            self.page_down();
                        }
                        KeyCode::Char('a') => {
                            if self.stage == SelectionStage::Model {
                                self.stage = SelectionStage::AddModel;
                                self.selected_index = 0;
                                self.list_state.select(Some(0));
                            }
                        }
                        KeyCode::Char('s') => {
                            if self.stage == SelectionStage::Server {
                                self.stage = SelectionStage::AddServer;
                                self.selected_index = 0;
                                self.list_state.select(Some(0));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn handle_enter(&mut self) -> bool {
        match self.stage {
            SelectionStage::Server => {
                if self.selected_index < self.servers.len() {
                    self.stage = SelectionStage::Model;
                    self.selected_index = 0;
                    self.list_state.select(Some(0));
                    return true;
                }
            }
            SelectionStage::Model => {
                let model_name = {
                    let model = match self.models.get(self.selected_index) {
                        Some(m) => m,
                        None => return true,
                    };
                    model.name.clone()
                };

                let needs_config = !self.config.models.contains_key(&model_name);
                if needs_config {
                    self.add_new_model(&model_name);
                }

                let model = self.models.get(self.selected_index);
                if let Some(m) = model {
                    self.selected_model = Some(m.clone());
                }
                return false;
            }
            SelectionStage::AddModel => {
                if self.selected_index < self.models.len() {
                    let model_name = self.models[self.selected_index].name.clone();
                    self.add_new_model(&model_name);
                    self.stage = SelectionStage::Model;
                    return true;
                }
            }
            SelectionStage::AddServer => {
                if self.selected_index < self.servers.len() {
                    self.add_new_server();
                    self.stage = SelectionStage::Server;
                    return true;
                }
            }
        }
        true
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(frame.area());

        let (title, items) = match self.stage {
            SelectionStage::Server => {
                let items: Vec<ListItem> = self
                    .servers
                    .iter()
                    .enumerate()
                    .map(|(idx, server)| {
                        let text = format!("{} - {}", server.name, server.path);
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
                (
                    " Select Llama Server (↑↓ to navigate, Enter to continue, 's' to add) ",
                    items,
                )
            }
            SelectionStage::Model => {
                let items: Vec<ListItem> = self
                    .models
                    .iter()
                    .enumerate()
                    .map(|(idx, model)| {
                        let quant = extract_quantization(&model.name);
                        let model_config = self.config.get_model_config(&model.name);
                        let server_name = model_config
                            .llama_server
                            .as_deref()
                            .unwrap_or(&self.config.default_llama_server);
                        let text = format!(
                            "{} - {} [{}] - Server: {}",
                            model.name,
                            format_size(model.size),
                            quant,
                            server_name
                        );
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
                (
                    " Select Model (↑↓ to navigate, Enter to launch, 'a' to add new) ",
                    items,
                )
            }
            SelectionStage::AddModel => {
                let items: Vec<ListItem> = self
                    .models
                    .iter()
                    .enumerate()
                    .map(|(idx, model)| {
                        let text = format!("{} - {}", model.name, format_size(model.size));
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
                (
                    " Select model to add config for (↑↓ to navigate, Enter to confirm) ",
                    items,
                )
            }
            SelectionStage::AddServer => {
                let items: Vec<ListItem> = self
                    .servers
                    .iter()
                    .enumerate()
                    .map(|(idx, server)| {
                        let text = format!("{} - {}", server.name, server.path);
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
                (
                    " Select server to edit (↑↓ to navigate, Enter to confirm) ",
                    items,
                )
            }
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, chunks[0], &mut self.list_state);

        let details_text = match self.stage {
            SelectionStage::Server => {
                if let Some(server) = self.servers.get(self.selected_index) {
                    format!(
                        "
Server Name: {}
Server Path: {}

Default Server: {}

Model Defaults:
  ctx_size: {:?}
  additional_args: {:?}
",
                        server.name,
                        server.path,
                        self.config.default_llama_server,
                        self.config.model_defaults.ctx_size,
                        self.config.model_defaults.additional_args
                    )
                } else {
                    "No server selected".to_string()
                }
            }
            SelectionStage::Model => {
                if let Some(selected_model) = self.models.get(self.selected_index) {
                    let model_config = self.config.get_model_config(&selected_model.name);
                    let server_name = model_config
                        .llama_server
                        .as_deref()
                        .unwrap_or(&self.config.default_llama_server);

                    format!(
                        "
Name: {}
Path: {}
Size: {}
Quantization: {}
MMProj: {}
Server: {}
ctx_size: {:?}
additional_args: {:?}
",
                        selected_model.name,
                        selected_model.gguf_path.display(),
                        format_size(selected_model.size),
                        extract_quantization(&selected_model.name),
                        selected_model
                            .mmproj_path
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| "Not found".to_string()),
                        server_name,
                        model_config.ctx_size,
                        model_config.additional_args
                    )
                } else {
                    "No model selected".to_string()
                }
            }
            SelectionStage::AddModel => {
                if self.selected_index < self.models.len() {
                    let model = &self.models[self.selected_index];
                    format!(
                        "
Name: {}
Path: {}
Size: {}

Press Enter to add this model with default configuration.
",
                        model.name,
                        model.gguf_path.display(),
                        format_size(model.size)
                    )
                } else {
                    "No model selected".to_string()
                }
            }
            SelectionStage::AddServer => {
                if self.selected_index < self.servers.len() {
                    let server = &self.servers[self.selected_index];
                    format!(
                        "
Server Name: {}
Server Path: {}

Press Enter to edit this server.
",
                        server.name, server.path
                    )
                } else {
                    "No server selected".to_string()
                }
            }
        };

        let details = Paragraph::new(details_text)
            .block(Block::default().borders(Borders::ALL).title(" Details "))
            .wrap(Wrap { trim: false });

        frame.render_widget(details, chunks[1]);
    }

    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    fn move_down(&mut self) {
        let max_index = match self.stage {
            SelectionStage::Server | SelectionStage::AddServer => {
                self.servers.len().saturating_sub(1)
            }
            SelectionStage::Model | SelectionStage::AddModel => self.models.len().saturating_sub(1),
        };
        if self.selected_index + 1 <= max_index {
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
        let max_index = match self.stage {
            SelectionStage::Server | SelectionStage::AddServer => {
                self.servers.len().saturating_sub(1)
            }
            SelectionStage::Model | SelectionStage::AddModel => self.models.len().saturating_sub(1),
        };
        if self.selected_index + page_size <= max_index {
            self.selected_index += page_size;
        } else {
            self.selected_index = max_index;
        }
        self.list_state.select(Some(self.selected_index));
    }

    fn add_new_model(&mut self, model_name: &str) {
        let cache_dir = crate::cache::get_cache_dir();
        let config_path = cache_dir.join("llama_sel_params.yaml");

        let mut new_config = self.config.model_defaults.clone();

        if let Some(existing_config) = self.config.models.get(model_name) {
            if existing_config.llama_server.is_some() {
                new_config.llama_server = existing_config.llama_server.clone();
            }
            if existing_config.ctx_size.is_some() {
                new_config.ctx_size = existing_config.ctx_size;
            }
            if existing_config.additional_args.is_some() {
                new_config.additional_args = existing_config.additional_args.clone();
            }
        }

        if let Ok(()) = add_model_config(&config_path, model_name, &new_config) {
            println!("Added model config for: {}", model_name);
            self.config
                .models
                .insert(model_name.to_string(), new_config);
        }
    }

    fn add_new_server(&mut self) {
        let cache_dir = crate::cache::get_cache_dir();
        let config_path = cache_dir.join("llama_sel_params.yaml");

        if self.selected_index < self.servers.len() {
            let server = &self.servers[self.selected_index];

            let counter = self
                .servers
                .iter()
                .filter(|s| s.name.starts_with(&server.name))
                .count();
            let new_name = format!("{}-{}", server.name, counter + 1);

            let new_server = LlamaServer {
                name: new_name,
                path: server.path.clone(),
            };

            if let Ok(()) = add_llama_server(&config_path, &new_server) {
                println!("Added new server: {}", new_server.name);
                self.servers.push(new_server.clone());
                self.config.llama_servers.push(new_server);
            }
        }
    }
}
