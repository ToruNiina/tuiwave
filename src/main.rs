mod timeseries;
mod load_vcd;
mod app;
mod ui;
mod log;

use app::TuiWave;

use crossterm::ExecutableCommand;
use crossterm::event::{
    Event, KeyEventKind
};

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
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("usage: ./tuiwave [filename.vcd]");
        return Err(anyhow::anyhow!("missing file"));
    }

    startup()?;

    let mut terminal = ratatui::terminal::Terminal::new(
        ratatui::backend::CrosstermBackend::new(std::io::stdout()))?;
    terminal.clear()?;

    let f = std::fs::File::open(&args[1])?;
    let ts = load_vcd::load_vcd(std::io::BufReader::new(f))?;

    let mut app = TuiWave::new(ts);
    app.setup_with_terminal_size(terminal.size()?);

    loop {
        update(&mut app)?;

        terminal.draw(|frame| { ui::draw_ui(&app, frame) })?;

        if app.should_quit {
            break;
        }
    }

    shutdown()?;

    return Ok(());
}
