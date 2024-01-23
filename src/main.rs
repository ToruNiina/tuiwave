mod timeseries;
mod load_vcd;

use crate::timeseries::*;
use crate::load_vcd::*;

use crossterm::ExecutableCommand;
use crossterm::event::{
    Event, KeyEventKind, KeyCode
};
use ratatui::style::{Style, Color};
use ratatui::text::{Line, Span};

use std::env;

#[derive(Debug, Clone)]
struct RuntimeError {
    msg: std::string::String,
}
impl RuntimeError {
    fn new(msg: String) -> Self {
        RuntimeError{msg}
    }
}
impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RuntimeError: msg = {}", self.msg)
    }
}
impl std::error::Error for RuntimeError {
    fn description(&self) -> &str {
        &self.msg
    }
}

fn format_time_series(name: String, timeline: &ValueChangeStream, t_from: u64, t_to: u64) -> Line {

    let mut current_t = t_from;
    let mut current_v = Value::Bits(Bits::Z);

    if let Some(before_start) = timeline.change_before(t_from) {
        current_v = timeline.history[before_start].new_value.clone();
    }
    let change_from = timeline.change_after(t_from);
    let change_to   = timeline.change_after(t_to  );

    // eprintln!("format_time_series: change_from = {:?}, to = {:?}", change_from, change_to);

    let mut spans = Vec::new();
    spans.push(Span::styled(name, Style::new().fg(Color::White).bg(Color::Black)));

    let style_bit = Style::new().fg(Color::LightGreen).bg(Color::Black);
    let style_var = Style::new().fg(Color::Black).bg(Color::LightGreen);
    let style_bad = Style::new().fg(Color::Black).bg(Color::LightRed);

    if let Some(change_from) = change_from {
        let change_to = change_to.unwrap_or(timeline.history.len().saturating_sub(1));

        for i in change_from..change_to {
            let mut currently_bad = false; // Z or X

            let change = &timeline.history[i];

            let dt = (change.time - current_t) as usize;
            assert!(0 != dt);

            let (txt, sty) = match current_v {
                Value::Bits(bits) => {
                    match bits {
                        Bits::B(x) => {
                            if x {
                                ("▔".repeat(4 * dt - 2), style_bit)
                            } else {
                                ("▁".repeat(4 * dt - 2), style_bit)
                            }
                        }
                        Bits::V(x) => {
                            (format!("{:<width$x}", x.value, width = 4*dt - 2), style_var)
                        }
                        Bits::X => {
                            currently_bad = true;
                            (format!("{:<width$}", "X", width = 4*dt - 2), style_bad)
                        }
                        Bits::Z => {
                            currently_bad = true;
                            (format!("{:<width$}", "Z", width = 4*dt - 2), style_bad)
                        }
                    }
                }
                Value::Real(x) => {
                    (format!("{:<width$}", x, width=4*dt-2), style_var)
                }
                Value::String(ref x) => {
                    (format!("{:<width$}", x, width=4*dt-2), style_var)
                }
            };
            spans.push(Span::styled(txt, sty));

            match change.new_value {
                Value::Bits(bits) => {
                    match bits {
                        Bits::B(x) => {
                            if x {
                                spans.push(Span::styled("▁".to_string(), style_bit));
                            } else {
                                spans.push(Span::styled("▁".to_string(), style_bit));
                            }
                        }
                        Bits::V(_) => {
                            spans.push(Span::styled("".to_string(),
                                Style::new().fg(Color::LightGreen).bg(Color::Black)
                            ));
                        }
                        Bits::X => {
                            if currently_bad {
                                spans.push(Span::styled("".to_string(),
                                    Style::new().fg(Color::LightRed).bg(Color::Black)
                                ));
                            } else {
                                spans.push(Span::styled("".to_string(),
                                    Style::new().fg(Color::LightGreen).bg(Color::Black)
                                ));
                                spans.push(Span::styled("".to_string(),
                                    Style::new().fg(Color::LightRed).bg(Color::Black)
                                ));
                            }
                        }
                        Bits::Z => {
                            if currently_bad {
                                spans.push(Span::styled("".to_string(),
                                    Style::new().fg(Color::LightRed).bg(Color::Black)
                                ));
                            } else {
                                spans.push(Span::styled("".to_string(),
                                    Style::new().fg(Color::LightGreen).bg(Color::Black)
                                ));
                                spans.push(Span::styled("".to_string(),
                                    Style::new().fg(Color::LightRed).bg(Color::Black)
                                ));
                            }
                        }
                    }
                }
                Value::Real(_) => {
                    spans.push(Span::styled("".to_string(),
                        Style::new().fg(Color::LightRed).bg(Color::Black)
                    ));
                }
                Value::String(_) => {
                    spans.push(Span::styled("".to_string(),
                        Style::new().fg(Color::LightRed).bg(Color::Black)
                    ));
                }
            }
            current_v = change.new_value.clone();
            current_t = change.time;
        }
    }

    if current_t < t_to {
        let dt = (t_to - current_t) as usize;
        let (txt, sty) = match current_v {
            Value::Bits(bits) => {
                match bits {
                    Bits::B(x) => {
                        if x {
                            ("▔".repeat(4 * dt), style_bit)
                        } else {
                            ("▁".repeat(4 * dt), style_bit)
                        }
                    }
                    Bits::V(x) => {
                        (format!("{:<width$x}", x.value, width=4*dt), style_var)
                    }
                    Bits::X => {
                        (format!("{:<width$}", "X", width=4*dt), style_bad)
                    }
                    Bits::Z => {
                        (format!("{:<width$}", "Z", width=4*dt), style_bad)
                    }
                }
            }
            Value::Real(x) => {
                (format!("{:<4}", x), style_var)
            }
            Value::String(ref x) => {
                (format!("{:<4}", x), style_var)
            }
        };
        spans.push(Span::styled(txt, sty));
    }
    ratatui::text::Line::from(spans)
}

fn show_values<'a>(ts: &'a TimeSeries, s: &'a Scope, t_from: u64, t_to: u64) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    for item in s.items.iter() {
        match item {
            ScopeItem::Value(v) => {
                lines.push(Line::from(""));
                lines.push(format_time_series(
                    format!("{:20}", v.name),
                    &ts.values[v.index], t_from, t_to
                ));
            }
            ScopeItem::Scope(_) => {
                // do nothing
            }
        }
    }
    for item in s.items.iter() {
        match item {
            ScopeItem::Value(_) => {
                // do nothing
            }
            ScopeItem::Scope(subscope) => {
                let ls = show_values(ts, subscope, t_from, t_to);
                lines.extend(ls.into_iter());
            }
        }
    }
    lines
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: ./tuiwave [filename.vcd]");
        return Ok(());
    }

    let f = std::fs::File::open(&args[1])?;
    let ts = load_vcd(std::io::BufReader::new(f))?;

    let mut t_last = 0;
    for vs in ts.values.iter() {
        for change in vs.history.iter() {
            if t_last < change.time {
                t_last = change.time;
            }
        }
    }
    // print_values(&ts, &ts.scope, 0, t_last + 1);

    std::io::stdout().execute(crossterm::terminal::EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    let mut terminal = ratatui::terminal::Terminal::new(
        ratatui::backend::CrosstermBackend::new(std::io::stdout()))?;
    terminal.clear()?;

    let mut resolution = 1;
    let mut t_from = 0;
    let mut t_to = t_last+1;

    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(
                ratatui::widgets::Paragraph::new(show_values(&ts, &ts.scope, t_from, t_to.min(t_last+1))),
                area,
            );
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(16)).unwrap() {
            if let Event::Key(key) = crossterm::event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Char('q') {
                        break;
                    } else if key.code == KeyCode::Char('l') || key.code == KeyCode::Right {
                        t_from = t_from.saturating_add(resolution);
                        t_to   = t_to  .saturating_add(resolution);
                    } else if key.code == KeyCode::Char('h') || key.code == KeyCode::Left {
                        if t_from != 0 {
                            t_from = t_from.saturating_sub(resolution);
                            t_to   = t_to  .saturating_sub(resolution);
                        }
                    } else if key.code == KeyCode::Char('+') {
                        resolution += 1;
                    } else if key.code == KeyCode::Char('-') {
                        if 1 < resolution {
                            resolution -= 1;
                        }
                    }
                }
            }
        }
    }

    std::io::stdout().execute(crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    return Ok(());
}
