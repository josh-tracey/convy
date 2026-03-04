use crate::lexer::Config;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

#[derive(Debug, Clone, PartialEq)]
enum Step {
    Type,
    Scope,
    IsBreaking,
    Description,
    Body,
    BreakingFooter,
    FooterPrompt,
    FooterKey,
    FooterValue,
    Confirm,
}

struct App {
    step: Step,
    config: Config,
    types: Vec<String>,
    type_state: ListState,
    
    scope_options: Option<Vec<String>>,
    scope_state: ListState,
    scope_input: String,
    
    is_breaking: bool,
    description_input: String,
    body_input: String,
    breaking_footer_input: String,
    
    footers: Vec<(String, String)>,
    footer_key_input: String,
    footer_value_input: String,
    add_footer_confirm: bool,
}

impl App {
    fn new(config: Config) -> App {
        let mut types = vec![
            "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert"
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
        
        if let Some(extra) = &config.additional_types {
            for t in extra {
                if !types.contains(t) {
                    types.push(t.clone());
                }
            }
        }
        types.sort();

        let mut type_state = ListState::default();
        type_state.select(Some(0));

        let scope_options = config.scopes.clone().map(|mut s| {
            s.insert(0, "(none)".to_string());
            s
        });
        
        let mut scope_state = ListState::default();
        if scope_options.is_some() {
            scope_state.select(Some(0));
        }

        App {
            step: Step::Type,
            config,
            types,
            type_state,
            scope_options,
            scope_state,
            scope_input: String::new(),
            is_breaking: false,
            description_input: String::new(),
            body_input: String::new(),
            breaking_footer_input: String::new(),
            footers: Vec::new(),
            footer_key_input: String::new(),
            footer_value_input: String::new(),
            add_footer_confirm: false,
        }
    }

    fn next_step(&mut self) {
        match self.step {
            Step::Type => self.step = Step::Scope,
            Step::Scope => self.step = Step::IsBreaking,
            Step::IsBreaking => self.step = Step::Description,
            Step::Description => self.step = Step::Body,
            Step::Body => {
                if self.is_breaking {
                    self.step = Step::BreakingFooter;
                } else {
                    self.step = Step::FooterPrompt;
                }
            }
            Step::BreakingFooter => self.step = Step::FooterPrompt,
            Step::FooterPrompt => {
                if self.add_footer_confirm {
                    self.step = Step::FooterKey;
                } else {
                    self.step = Step::Confirm;
                }
            }
            Step::FooterKey => {
                if !self.footer_key_input.trim().is_empty() {
                    self.step = Step::FooterValue;
                }
            }
            Step::FooterValue => {
                if !self.footer_value_input.trim().is_empty() {
                    self.footers.push((self.footer_key_input.clone(), self.footer_value_input.clone()));
                    self.footer_key_input.clear();
                    self.footer_value_input.clear();
                    self.add_footer_confirm = false;
                    self.step = Step::FooterPrompt;
                }
            }
            Step::Confirm => {}
        }
    }

    fn prev_step(&mut self) {
        match self.step {
            Step::Type => {}
            Step::Scope => self.step = Step::Type,
            Step::IsBreaking => self.step = Step::Scope,
            Step::Description => self.step = Step::IsBreaking,
            Step::Body => self.step = Step::Description,
            Step::BreakingFooter => self.step = Step::Body,
            Step::FooterPrompt => {
                if self.is_breaking {
                    self.step = Step::BreakingFooter;
                } else {
                    self.step = Step::Body;
                }
            }
            Step::FooterKey => self.step = Step::FooterPrompt,
            Step::FooterValue => self.step = Step::FooterKey,
            Step::Confirm => self.step = Step::FooterPrompt,
        }
    }

    fn construct_message(&self) -> String {
        let commit_type = &self.types[self.type_state.selected().unwrap_or(0)];
        let scope = if let Some(options) = &self.scope_options {
            let sel = &options[self.scope_state.selected().unwrap_or(0)];
            if sel == "(none)" { None } else { Some(sel.clone()) }
        } else {
            if self.scope_input.trim().is_empty() { None } else { Some(self.scope_input.trim().to_string()) }
        };

        let mut msg = String::new();
        msg.push_str(commit_type);
        if let Some(s) = scope {
            msg.push('(');
            msg.push_str(&s);
            msg.push(')');
        }
        if self.is_breaking {
            msg.push('!');
        }
        msg.push_str(": ");

        if self.config.emoji.unwrap_or(false) {
            let emoji = match commit_type.as_str() {
                "feat" => "✨ ",
                "fix" => "🐛 ",
                "docs" => "📚 ",
                "style" => "💎 ",
                "refactor" => "♻️ ",
                "perf" => "🚀 ",
                "test" => "🚨 ",
                "build" => "🛠 ",
                "ci" => "⚙️ ",
                "chore" => "🔧 ",
                "revert" => "⏪ ",
                _ => "",
            };
            msg.push_str(emoji);
        }

        msg.push_str(&self.description_input);

        if !self.body_input.trim().is_empty() {
            msg.push_str("\n\n");
            msg.push_str(&self.body_input);
        }

        if self.is_breaking && !self.breaking_footer_input.trim().is_empty() {
            msg.push_str("\n\nBREAKING CHANGE: ");
            msg.push_str(&self.breaking_footer_input);
        }

        // Add footers
        if !self.footers.is_empty() {
             if self.body_input.trim().is_empty() && (!self.is_breaking || self.breaking_footer_input.trim().is_empty()) {
                msg.push_str("\n\n");
            } else if !msg.ends_with('\n') {
                msg.push('\n');
            }

            for (k, v) in &self.footers {
                msg.push_str(&format!("{}: {}\n", k, v));
            }
            if msg.ends_with('\n') {
                msg.pop();
            }
        }

        msg
    }
}

pub fn run_wizard(config: Config) -> Result<Option<String>, io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new(config);
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Ok(true) = res {
        Ok(Some(app.construct_message()))
    } else {
        Ok(None)
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app)).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        if let Event::Key(key) = event::read()? {
            match app.step {
                Step::Type => match key.code {
                    KeyCode::Up => {
                        let i = match app.type_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    app.types.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.type_state.select(Some(i));
                    }
                    KeyCode::Down => {
                        let i = match app.type_state.selected() {
                            Some(i) => {
                                if i >= app.types.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        app.type_state.select(Some(i));
                    }
                    KeyCode::Enter => app.next_step(),
                    KeyCode::Esc => return Ok(false),
                    _ => {}
                },
                Step::Scope => {
                    if let Some(options) = &app.scope_options {
                        match key.code {
                            KeyCode::Up => {
                                let i = match app.scope_state.selected() {
                                    Some(i) => {
                                        if i == 0 {
                                            options.len() - 1
                                        } else {
                                            i - 1
                                        }
                                    }
                                    None => 0,
                                };
                                app.scope_state.select(Some(i));
                            }
                            KeyCode::Down => {
                                let i = match app.scope_state.selected() {
                                    Some(i) => {
                                        if i >= options.len() - 1 {
                                            0
                                        } else {
                                            i + 1
                                        }
                                    }
                                    None => 0,
                                };
                                app.scope_state.select(Some(i));
                            }
                            KeyCode::Enter => app.next_step(),
                            KeyCode::Backspace => app.prev_step(),
                            KeyCode::Esc => return Ok(false),
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char(c) => app.scope_input.push(c),
                            KeyCode::Backspace => {
                                if app.scope_input.is_empty() {
                                    app.prev_step();
                                } else {
                                    app.scope_input.pop();
                                }
                            }
                            KeyCode::Enter => app.next_step(),
                            KeyCode::Esc => return Ok(false),
                            _ => {}
                        }
                    }
                }
                Step::IsBreaking => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Right | KeyCode::Char(' ') => {
                        app.is_breaking = true;
                        app.next_step();
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Left => {
                        app.is_breaking = false;
                        app.next_step();
                    }
                    KeyCode::Enter => app.next_step(),
                    KeyCode::Backspace => app.prev_step(),
                    KeyCode::Esc => return Ok(false),
                    _ => {}
                },
                Step::Description => match key.code {
                    KeyCode::Char(c) => app.description_input.push(c),
                    KeyCode::Backspace => {
                        if app.description_input.is_empty() {
                            app.prev_step();
                        } else {
                            app.description_input.pop();
                        }
                    }
                    KeyCode::Enter => {
                        if !app.description_input.trim().is_empty() && app.description_input.len() <= 100 {
                            app.next_step();
                        }
                    }
                    KeyCode::Esc => return Ok(false),
                    _ => {}
                },
                Step::Body => match key.code {
                    KeyCode::Char(c) => app.body_input.push(c),
                    KeyCode::Backspace => {
                        if app.body_input.is_empty() {
                            app.prev_step();
                        } else {
                            app.body_input.pop();
                        }
                    }
                    KeyCode::Enter => app.next_step(),
                    KeyCode::Esc => return Ok(false),
                    _ => {}
                },
                Step::BreakingFooter => match key.code {
                    KeyCode::Char(c) => app.breaking_footer_input.push(c),
                    KeyCode::Backspace => {
                        if app.breaking_footer_input.is_empty() {
                            app.prev_step();
                        } else {
                            app.breaking_footer_input.pop();
                        }
                    }
                    KeyCode::Enter => app.next_step(),
                    KeyCode::Esc => return Ok(false),
                    _ => {}
                },
                Step::FooterPrompt => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Right | KeyCode::Char(' ') => {
                        app.add_footer_confirm = true;
                        app.next_step();
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Left | KeyCode::Enter => {
                        app.add_footer_confirm = false;
                        app.next_step();
                    }
                    KeyCode::Backspace => app.prev_step(),
                    KeyCode::Esc => return Ok(false),
                    _ => {}
                }
                Step::FooterKey => match key.code {
                    KeyCode::Char(c) => app.footer_key_input.push(c),
                    KeyCode::Backspace => {
                        if app.footer_key_input.is_empty() {
                            app.prev_step();
                        } else {
                            app.footer_key_input.pop();
                        }
                    }
                    KeyCode::Enter => app.next_step(),
                    KeyCode::Esc => return Ok(false),
                    _ => {}
                }
                Step::FooterValue => match key.code {
                    KeyCode::Char(c) => app.footer_value_input.push(c),
                    KeyCode::Backspace => {
                        if app.footer_value_input.is_empty() {
                            app.prev_step();
                        } else {
                            app.footer_value_input.pop();
                        }
                    }
                    KeyCode::Enter => app.next_step(),
                    KeyCode::Esc => return Ok(false),
                    _ => {}
                }
                Step::Confirm => match key.code {
                    KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(true),
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => return Ok(false),
                    KeyCode::Backspace => app.prev_step(),
                    _ => {}
                },
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.area());

    let title = Paragraph::new("Convy Commit Wizard")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    match app.step {
        Step::Type => {
            let items: Vec<ListItem> = app
                .types
                .iter()
                .map(|t| ListItem::new(t.as_str()))
                .collect();
            let list = List::new(items)
                .block(Block::default().title("Select Type").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, chunks[1], &mut app.type_state);
        }
        Step::Scope => {
            if let Some(options) = &app.scope_options {
                let items: Vec<ListItem> = options
                    .iter()
                    .map(|t| ListItem::new(t.as_str()))
                    .collect();
                let list = List::new(items)
                    .block(Block::default().title("Select Scope").borders(Borders::ALL))
                    .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                    .highlight_symbol(">> ");
                f.render_stateful_widget(list, chunks[1], &mut app.scope_state);
            } else {
                let input = Paragraph::new(app.scope_input.as_str())
                    .block(Block::default().title("Scope (optional)").borders(Borders::ALL));
                f.render_widget(input, chunks[1]);
            }
        }
        Step::IsBreaking => {
            let msg = if app.is_breaking { "YES (Press 'n' to change)" } else { "NO (Press 'y' to change)" };
            let p = Paragraph::new(msg)
                .block(Block::default().title("Is this a breaking change?").borders(Borders::ALL));
            f.render_widget(p, chunks[1]);
        }
        Step::Description => {
            let mut style = Style::default();
            if app.description_input.len() > 100 {
                style = style.fg(Color::Red);
            }
            let input = Paragraph::new(app.description_input.as_str())
                .style(style)
                .block(Block::default().title(format!("Description ({}/100)", app.description_input.len())).borders(Borders::ALL));
            f.render_widget(input, chunks[1]);
        }
        Step::Body => {
            let input = Paragraph::new(app.body_input.as_str())
                .block(Block::default().title("Body (optional)").borders(Borders::ALL));
            f.render_widget(input, chunks[1]);
        }
        Step::BreakingFooter => {
            let input = Paragraph::new(app.breaking_footer_input.as_str())
                .block(Block::default().title("Breaking Change Description").borders(Borders::ALL));
            f.render_widget(input, chunks[1]);
        }
        Step::FooterPrompt => {
            let msg = if app.add_footer_confirm { "YES (Press 'n' to change)" } else { "NO (Press 'y' to change)" };
            let p = Paragraph::new(msg)
                .block(Block::default().title("Add a footer?").borders(Borders::ALL));
            f.render_widget(p, chunks[1]);
        }
        Step::FooterKey => {
            let input = Paragraph::new(app.footer_key_input.as_str())
                .block(Block::default().title("Footer Key (e.g. Issue)").borders(Borders::ALL));
            f.render_widget(input, chunks[1]);
        }
        Step::FooterValue => {
            let input = Paragraph::new(app.footer_value_input.as_str())
                .block(Block::default().title("Footer Value (e.g. #123)").borders(Borders::ALL));
            f.render_widget(input, chunks[1]);
        }
        Step::Confirm => {
            let preview = app.construct_message();
            let p = Paragraph::new(preview)
                .block(Block::default().title("Preview (Press Enter to Commit, Esc to Cancel)").borders(Borders::ALL));
            f.render_widget(p, chunks[1]);
        }
    }

    let footer = Paragraph::new("Enter: Next | Backspace: Prev | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[2]);
}
