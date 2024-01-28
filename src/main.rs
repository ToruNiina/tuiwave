mod timeseries;
mod load_vcd;
mod app;

use app::TuiWave;

use crossterm::ExecutableCommand;
use crossterm::event::{
    Event, KeyEventKind, KeyCode
};
use ratatui::terminal::Frame;
use ratatui::layout::{Layout, Constraint, Direction};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::symbols;
use ratatui::style::{Style, Color};

use std::env;

fn draw_ui(app: &app::TuiWave, frame: &mut Frame) {

    let lines = app::show_values(&app, &app.ts.scope);

    let area = frame.size();
    let n_lines = area.height as usize / 2;

    // the first line has all (including top and bottom) borders so takes 3 lines.
    let mut constraints = vec![Constraint::Length(3)];
    // other lines does not have top border. takes 2 lines.
    constraints.extend(Constraint::from_lengths(std::iter::repeat(2).take(n_lines-1)));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints) // Constraint::from_lengths(std::iter::repeat(2).take(n_lines)))
        .split(frame.size());

    for i in 0..n_lines {

        let (border_set, borders) = if i == 0 {
            (
                symbols::border::Set {
                    bottom_left: symbols::line::NORMAL.vertical_right, // |-
                    bottom_right: symbols::line::NORMAL.vertical_left, // -|
                    .. symbols::border::PLAIN
                },
                Borders::ALL
            )
        } else if i+1 == n_lines || i+1 == lines.len() {
            (
                symbols::border::Set {.. symbols::border::PLAIN},
                Borders::BOTTOM | Borders::LEFT | Borders::RIGHT
            )
        } else {
            (
                symbols::border::Set {
                    bottom_left: symbols::line::NORMAL.vertical_right, // |-
                    bottom_right: symbols::line::NORMAL.vertical_left, // -|
                    .. symbols::border::PLAIN
                },
                Borders::BOTTOM | Borders::LEFT | Borders::RIGHT
            )
        };

        if i < lines.len() {
            frame.render_widget(
                Paragraph::new(lines[i].clone())
                    .block(Block::new().borders(borders).border_set(border_set)
                        .border_style(Style::new().fg(Color::DarkGray))
                        ),
                layout[i]
            );
        } else {
            frame.render_widget(
                Block::new().borders(Borders::NONE),
                layout[i]
            );
        }
    }
}

fn update(app: &mut TuiWave) -> anyhow::Result<()> {
    if crossterm::event::poll(std::time::Duration::from_millis(250))? {
        if let Event::Key(key) = crossterm::event::read()? {
            if key.kind == KeyEventKind::Press {
                if key.code == KeyCode::Char('q') {
                    app.should_quit = true;
                } else if key.code == KeyCode::Char('l') || key.code == KeyCode::Right {
                    app.t_from = app.t_from.saturating_add(1);
                    app.t_to   = app.t_to  .saturating_add(1);
                } else if key.code == KeyCode::Char('h') || key.code == KeyCode::Left {
                    if app.t_from != 0 {
                        app.t_from = app.t_from.saturating_sub(1);
                        app.t_to   = app.t_to  .saturating_sub(1);
                    }
                } else if key.code == KeyCode::Char('-') {
                    app.width = app.width.saturating_sub(1).max(2);
                } else if key.code == KeyCode::Char('+') {
                    app.width = app.width.saturating_add(1).max(2);
                }
            }
        }
    }
    Ok(())
}

fn startup() -> anyhow::Result<()> {
    std::io::stdout().execute(crossterm::terminal::EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    Ok(())
}

fn shutdown() -> anyhow::Result<()> {
    std::io::stdout().execute(crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: ./tuiwave [filename.vcd]");
        return Err(anyhow::anyhow!("missing file"));
    }

    let f = std::fs::File::open(&args[1])?;
    let ts = load_vcd::load_vcd(std::io::BufReader::new(f))?;
    let mut app = TuiWave::new(ts);

    startup()?;

    let mut terminal = ratatui::terminal::Terminal::new(
        ratatui::backend::CrosstermBackend::new(std::io::stdout()))?;
    terminal.clear()?;
    loop {
        update(&mut app)?;

        terminal.draw(|frame| { draw_ui(&app, frame) })?;

        if app.should_quit {
            break;
        }
    }

    shutdown()?;

    return Ok(());
}
