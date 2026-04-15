use anyhow::Result;
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

use crate::pattern::{Pattern, load_all};

enum InputMode { Normal, Searching }

struct App {
    patterns: Vec<Pattern>,
    filtered: Vec<usize>,
    list_state: ListState,
    mode: InputMode,
    query: String,
    scroll: u16,
    running: bool,
}

impl App {
    fn new(patterns: Vec<Pattern>) -> Self {
        let filtered = (0..patterns.len()).collect();
        Self {
            patterns,
            filtered,
            list_state: ListState::default().with_selected(Some(0)),
            mode: InputMode::Normal,
            query: String::new(),
            scroll: 0,
            running: true,
        }
    }

    fn handle_key(&mut self, code: KeyCode) {
        match self.mode {
            InputMode::Normal => match code {
                KeyCode::Char('q') => self.running = false,
                KeyCode::Char('j') | KeyCode::Down => { self.scroll = 0; self.list_state.select_next(); }
                KeyCode::Char('k') | KeyCode::Up => { self.scroll = 0; self.list_state.select_previous(); }
                KeyCode::Char('/') => self.mode = InputMode::Searching,
                KeyCode::PageDown => self.scroll = self.scroll.saturating_add(1),
                KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(1),
                _ => {}
            },
            InputMode::Searching => match code {
                KeyCode::Esc => { self.query.clear(); self.refilter(); self.mode = InputMode::Normal; }
                KeyCode::Enter => self.mode = InputMode::Normal,
                KeyCode::Backspace => { self.query.pop(); self.refilter(); }
                KeyCode::Char(c) => { self.query.push(c); self.refilter(); }
                _ => {}
            },
        }
    }

    fn refilter(&mut self) {
        let q = self.query.to_lowercase();
        self.filtered = self.patterns.iter().enumerate()
            .filter(|(_, p)| {
                q.is_empty()
                    || p.metadata.pattern.to_lowercase().contains(&q)
                    || p.metadata.category.to_lowercase().contains(&q)
                    || p.metadata.tags.iter().any(|t| t.to_lowercase().contains(&q))
                    || p.content.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect();
        self.list_state.select(if self.filtered.is_empty() { None } else { Some(0) });
        self.scroll = 0;
    }
}

pub fn run() -> Result<()> {
    let patterns = load_all()?;
    let mut terminal = ratatui::init();
    let mut app = App::new(patterns);

    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || loop {
        if event::poll(std::time::Duration::from_millis(100)).unwrap_or(false)
            && let Ok(event::Event::Key(k)) = event::read()
            && tx.send(k).is_err()
        {
            break;
        }
    });

    while app.running {
        terminal.draw(|f| draw(f, &mut app))?;
        if let Ok(k) = rx.recv_timeout(std::time::Duration::from_millis(50))
            && k.kind == KeyEventKind::Press
        {
            app.handle_key(k.code);
        }
    }

    ratatui::restore();
    Ok(())
}

fn draw(frame: &mut Frame, app: &mut App) {
    let [top, main, status] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ]).areas(frame.area());

    let [nav, content] = Layout::horizontal([
        Constraint::Length(30),
        Constraint::Min(0),
    ]).areas(main);

    // Title
    frame.render_widget(
        Paragraph::new("✦ Code Grimoire ✦")
            .alignment(Alignment::Center)
            .style(Style::new().blue().bold())
            .block(Block::bordered()),
        top,
    );

    // Nav list
    let items: Vec<ListItem> = app.filtered.iter()
        .map(|&i| ListItem::new(app.patterns[i].metadata.pattern.as_str()))
        .collect();
    let list = List::new(items)
        .block(Block::bordered().title("Patterns").title_style(Color::Yellow))
        .highlight_symbol(">> ")
        .highlight_style(Style::new().bg(Color::DarkGray).bold());
    frame.render_stateful_widget(list, nav, &mut app.list_state);

    // Content
    if app.filtered.is_empty() {
        frame.render_widget(
            Paragraph::new("No patterns found.")
                .block(Block::bordered().title("Content").title_style(Color::Cyan)),
            content,
        );
    } else {
        let p = &app.patterns[app.filtered[app.list_state.selected().unwrap_or(0)]];
        let mut lines = vec![
            Line::from(vec![Span::styled("Pattern:  ", Style::new().yellow()), Span::styled(&p.metadata.pattern, Style::new().cyan().bold())]),
            Line::from(vec![Span::styled("Category: ", Style::new().yellow()), Span::raw(&p.metadata.category)]),
        ];
        if let Some(fw) = &p.metadata.framework {
            lines.push(Line::from(vec![Span::styled("Framework:", Style::new().yellow()), Span::raw(format!(" {fw}"))]));
        }
        if !p.metadata.tags.is_empty() {
            lines.push(Line::from(vec![Span::styled("Tags:     ", Style::new().yellow()), Span::styled(p.metadata.tags.join(", "), Style::new().light_green())]));
        }
        lines.push(Line::from(""));
        for line in p.content.lines() {
            lines.push(Line::from(line.to_string()));
        }
        frame.render_widget(
            Paragraph::new(lines)
                .block(Block::bordered().title(p.metadata.pattern.as_str()).title_style(Color::Cyan))
                .scroll((app.scroll, 0)),
            content,
        );
    }

    // Status bar
    let status_text = match app.mode {
        InputMode::Normal => Paragraph::new("q: quit  j/k: navigate  /: search  PgUp/PgDn: scroll").alignment(Alignment::Center),
        InputMode::Searching => Paragraph::new(Line::from(vec![
            Span::styled("Search: ", Style::new().yellow()),
            Span::raw(&app.query),
            Span::raw("▌"),
        ])),
    };
    frame.render_widget(status_text, status);
}
