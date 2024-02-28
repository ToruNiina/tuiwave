mod timeseries;
mod load_vcd;
mod app;

use app::TuiWave;

use crossterm::ExecutableCommand;
use crossterm::event::{
    Event, KeyEventKind
};
use ratatui::terminal::Frame;
use ratatui::layout::{Layout, Constraint, Direction};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::symbols;
use ratatui::style::{Style, Color};

use std::env;

fn draw_ui(app: &app::TuiWave, frame: &mut Frame) {

    // add side bar showing a list of signals
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_percentages([15, 85]))
        .split(frame.size());

    let values = app::list_values(&app, &app.ts.scope, &app.ts.scope.name);

    let mut constraints = vec![Constraint::Length(3)];
    // other lines does not have top border. takes 2 lines.
    constraints.extend(Constraint::from_lengths(std::iter::repeat(2).take(values.len())));

    let sublayout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(layout[0]);

    //   .------------.
    //   | 1st signal |
    //   |------------|
    //
    //   | 2nd signal |
    //   |------------|
    //
    //   | Nth path   |
    //   '------------'

    let first_path_borders = Borders::TOP | Borders::BOTTOM | Borders::LEFT | Borders::RIGHT;
    let default_path_borders = Borders::BOTTOM | Borders::LEFT | Borders::RIGHT;

    let default_path_set = symbols::border::Set {
        bottom_left: "├",
        bottom_right: "┤",
        .. symbols::border::PLAIN
    };
    let last_path_set = symbols::border::Set {
        .. symbols::border::PLAIN
    };

    for (idx, (name, _)) in values.iter().enumerate() {
        let is_first = idx == 0;
        let is_last  = idx+1 == values.len();

        frame.render_widget(
            Paragraph::new(name.clone()).block(
                Block::new()
                .borders(if is_first {first_path_borders} else {default_path_borders})
                .border_set(if is_last {last_path_set} else {default_path_set})
                .border_style(Style::new().fg(Color::DarkGray))
            ),
            sublayout[idx]);
    }

    app::draw_timeline(app, frame, &layout[1]);
}

fn update(app: &mut TuiWave) -> anyhow::Result<()> {

    if crossterm::event::poll(std::time::Duration::from_millis(1000/60))? {
        match crossterm::event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    app.key_press(key.code, key.modifiers, key.state);
                }
            },
            Event::Resize(w, h) => {
                app.resize(w, h);
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

    let termsize = terminal.size()?;
    let n_lines = termsize.height as usize / 2;
    let n_lines = if termsize.height % 2 == 1 { n_lines } else { n_lines - 1 };
    app.current_drawable_lines = n_lines;

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
