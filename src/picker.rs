use crate::db::{self, Snippet};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use rusqlite::Connection;
use std::io;

struct State {
    snippets: Vec<Snippet>,
    query: String,
    selected: usize,
}

impl State {
    fn new(snippets: Vec<Snippet>) -> Self {
        Self { snippets, query: String::new(), selected: 0 }
    }

    fn filtered(&self) -> Vec<usize> {
        let q = self.query.to_lowercase();
        self.snippets
            .iter()
            .enumerate()
            .filter(|(_, s)| s.command.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect()
    }

    fn clamp(&mut self, filtered_len: usize) {
        if filtered_len == 0 {
            self.selected = 0;
        } else if self.selected >= filtered_len {
            self.selected = filtered_len - 1;
        }
    }
}

pub fn pick(conn: &Connection, snippets: Vec<Snippet>) -> anyhow::Result<Option<Snippet>> {
    if snippets.is_empty() {
        return Ok(None);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, snippets, conn);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    snippets: Vec<Snippet>,
    conn: &Connection,
) -> anyhow::Result<Option<Snippet>> {
    let mut state = State::new(snippets);
    let mut list_state = ListState::default();

    loop {
        let filtered = state.filtered();
        state.clamp(filtered.len());
        list_state.select(filtered.is_empty().then_some(None).unwrap_or(Some(state.selected)));

        terminal.draw(|f| render(f, &mut list_state, &state, &filtered))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Esc => return Ok(None),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(None);
                }
                KeyCode::Up => {
                    if state.selected > 0 {
                        state.selected -= 1;
                    }
                }
                KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE && state.query.is_empty() => {
                    if state.selected > 0 {
                        state.selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if state.selected + 1 < filtered.len() {
                        state.selected += 1;
                    }
                }
                KeyCode::Char('j') if key.modifiers == KeyModifiers::NONE && state.query.is_empty() => {
                    if state.selected + 1 < filtered.len() {
                        state.selected += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(&orig_idx) = filtered.get(state.selected) {
                        return Ok(Some(state.snippets.remove(orig_idx)));
                    }
                }
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(&orig_idx) = filtered.get(state.selected) {
                        let id = state.snippets[orig_idx].id;
                        db::delete(conn, id)?;
                        state.snippets.remove(orig_idx);
                        if state.snippets.is_empty() {
                            return Ok(None);
                        }
                    }
                }
                KeyCode::Backspace => {
                    state.query.pop();
                    state.selected = 0;
                }
                KeyCode::Char(c) => {
                    state.query.push(c);
                    state.selected = 0;
                }
                _ => {}
            }
        }
    }
}

fn render(f: &mut Frame, list_state: &mut ListState, state: &State, filtered: &[usize]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3), Constraint::Length(1)])
        .split(f.area());

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|&i| ListItem::new(Line::from(Span::raw(&state.snippets[i].command))))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" snip "))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[0], list_state);

    let search = Paragraph::new(format!("> {}", state.query))
        .block(Block::default().borders(Borders::ALL).title(" search "));
    f.render_widget(search, chunks[1]);

    let help = Paragraph::new("  enter: run   ctrl+d: delete   esc: quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}
