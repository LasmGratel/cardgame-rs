use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders},
    Frame, Terminal,
};
use tui::widgets::{Cell, Paragraph, Row, Table};
use crate::AppBuilder;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_ui.system());
    }
}

fn setup_ui(mut commands: Commands, thread_pool: Res<AsyncComputeTaskPool>) {
    let e: Result<(), Box<dyn Error>> = (|| {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let task = thread_pool.spawn(async move {
            let mut terminal = terminal;
            // create app and run it
            let res = run_app(&mut terminal);

            // restore terminal
            disable_raw_mode().unwrap();
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            ).unwrap();
            terminal.show_cursor().unwrap();

            if let Err(err) = res {
                eprintln!("{:?}", err)
            }
        });

        task.detach();

        Ok(())
    })();
    e.unwrap();
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    loop {
        terminal.draw(login_ui)?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => {
                    app.messages.push(app.input.drain(..).collect());
                }
                KeyCode::Char(c) => {
                    app.input.push(c);
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Esc => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

fn login_ui<B: Backend>(f: &mut Frame<B>) {
    let size = f.size();

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Login")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let login_block = Block::default()
        .title(
            Span::from("Login"),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default());

    let login_table = Table::new(vec![
        Row::new(vec![Cell::from("Username"), Cell::from("")]),
        Row::new(vec![Cell::from("Server"), Cell::from("")]),
    ]).block(login_block)
        .column_spacing(2)
        .widths(&[Constraint::Percentage(20), Constraint::Percentage(80)]);

    Paragraph::new();

    f.render_widget(login_table, chunks[0]);


}

fn ui<B: Backend>(f: &mut Frame<B>) {
    // Wrapping block for a group
    // Just draw the block and the group on the same area and build the group
    // with at least a margin of 1
    let size = f.size();

    // Surrounding block
    let block = Block::default()
        .borders(Borders::ALL)
        .title("斗地主")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    // Top two inner blocks
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);

    // Top left inner block with green background
    let block = Block::default()
        .title(vec![
            Span::styled("With", Style::default().fg(Color::Yellow)),
            Span::from(" background"),
        ])
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Green));
    f.render_widget(block, top_chunks[0]);

    // Top right inner block with styled title aligned to the right
    let block = Block::default()
        .title(Span::styled(
            "Styled title",
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Right);
    f.render_widget(block, top_chunks[1]);

    // Bottom two inner blocks
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    // Bottom left block with all default borders
    let block = Block::default().title("With borders").borders(Borders::ALL);
    f.render_widget(block, bottom_chunks[0]);

    // Bottom right block with styled left and right border
    let block = Block::default()
        .title("With styled borders and doubled borders")
        .border_style(Style::default().fg(Color::Cyan))
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_type(BorderType::Double);
    f.render_widget(block, bottom_chunks[1]);
}