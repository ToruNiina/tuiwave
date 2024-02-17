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

    let values = app::list_values(&app, &app.ts.scope, &app.ts.scope.name);
    let lines = app::format_values(&app, values);

    let area = frame.size();
    let n_lines = area.height as usize / 2;

    // the first line has all (including top and bottom) borders so takes 3 lines.
    let mut constraints = vec![Constraint::Length(3)];
    // other lines does not have top border. takes 2 lines.
    constraints.extend(Constraint::from_lengths(std::iter::repeat(2).take(n_lines)));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.size());

    // we have 3 kinds of borders for the first, the last, and the other blocks.
    //
    //   .---------- -------------.
    //   | 1st path  | 1st signal |
    //   |---------- +------------|
    //
    //   | 2nd path  | 2nd signal |
    //   |---------- +------------|
    //
    //   | Nth path  | Nth signal |
    //   '---------- -------------'
    //

    let first_path_borders = Borders::TOP | Borders::BOTTOM | Borders::LEFT;
    let first_sign_borders = Borders::ALL;

    let default_path_borders = Borders::BOTTOM | Borders::LEFT;
    let default_sign_borders = Borders::BOTTOM | Borders::LEFT | Borders::RIGHT;

    let default_path_set = symbols::border::Set {
        bottom_left: "├",
        .. symbols::border::PLAIN
    };
    let default_path_set_next_focused = symbols::border::Set {
        bottom_left: "┢",
        bottom_right: "┪",
        horizontal_bottom: "━",
        .. symbols::border::PLAIN
    };
    let default_path_set_focused = symbols::border::Set {
        bottom_left: "┡",
        .. symbols::border::THICK
    };

    let default_sign_set = symbols::border::Set {
        top_left: "┬",
        bottom_left: "┼",
        bottom_right: "┤",
        .. symbols::border::PLAIN
    };
    let default_sign_set_next_focused = symbols::border::Set {
        top_left: "┬",
        bottom_left: "╈",
        bottom_right: "┪",
        horizontal_bottom: "━",
        .. symbols::border::PLAIN
    };
    let default_sign_set_focused = symbols::border::Set {
        top_left: "┳",
        bottom_left: "╇",
        bottom_right: "┩",
        .. symbols::border::THICK
    };

    let last_path_set = symbols::border::Set {
        .. symbols::border::PLAIN
    };
    let last_path_set_focused = symbols::border::Set {
        .. symbols::border::THICK
    };

    let last_sign_set = symbols::border::Set {
        top_left: "┬",
        bottom_left: "┴",
        .. symbols::border::PLAIN
    };
    let last_sign_set_focused = symbols::border::Set {
        top_left: "┳",
        bottom_left: "┻",
        .. symbols::border::THICK
    };

    for i in 0..n_lines {

        let idx = i + app.line_from as usize;

        let is_first = i == 0;
        let is_last = i+1 == n_lines || idx+1 == lines.len();

        if idx < lines.len() {
            let (path, line) = &lines[idx];

            let sublayout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(Constraint::from_percentages([15, 85]))
                .split(layout[i]);

            let is_focused = idx == app.line_focused;
            let next_focused = !is_last && (idx+1) == app.line_focused;

            frame.render_widget(
                Paragraph::new(path.clone())
                    .block(
                        Block::new()
                        .borders(if is_first {first_path_borders} else {default_path_borders})
                        .border_set(if is_last {
                            if is_focused {last_path_set_focused} else {last_path_set}
                        } else {
                            if is_focused {default_path_set_focused} else if next_focused {default_path_set_next_focused} else {default_path_set}
                        })
                        .border_style(Style::new().fg(Color::DarkGray))
                    ),
                sublayout[0]
            );

            frame.render_widget(
                Paragraph::new(line.clone())
                    .block(
                        Block::new()
                            .borders(if is_first {first_sign_borders} else {default_sign_borders})
                            .border_set(if is_last {
                                if is_focused {last_sign_set_focused} else {last_sign_set}
                            } else {
                                if is_focused {default_sign_set_focused} else if next_focused {default_sign_set_next_focused} else {default_sign_set}
                            })
                            .border_style(Style::new().fg(Color::DarkGray))
                    ),
                sublayout[1]
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

    if crossterm::event::poll(std::time::Duration::from_millis(1000/60))? {
        match crossterm::event::read()? {
            Event::Key(key) => {
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
                    } else if key.code == KeyCode::Char('j') || key.code == KeyCode::Down {
                        app.line_focused = (app.line_focused + 1).min(app.ts.values.len().saturating_sub(1));
                    } else if key.code == KeyCode::Char('k') || key.code == KeyCode::Up {
                        app.line_focused = app.line_focused.saturating_sub(1);
                    } else if key.code == KeyCode::Char('-') {
                        app.width = app.width.saturating_sub(1).max(2);
                    } else if key.code == KeyCode::Char('+') {
                        app.width = app.width.saturating_add(1).max(2);
                    } else if key.code == KeyCode::Char('0') {
                        app.t_to   = app.t_to.saturating_sub(app.t_from);
                        app.t_from = 0;
                    }
                }
            },
            Event::Resize(_w, h) => {
                let n_lines = h as usize / 2;
                let n_lines = if h % 2 == 1 { n_lines } else { n_lines - 1 };

                if app.line_focused < app.line_from {
                    app.line_from = app.line_focused;
                }
                if (n_lines + app.line_from).saturating_sub(1) < app.line_focused {
                    app.line_from = app.line_focused - n_lines + 1;
                }

            },
            _ => {}
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
