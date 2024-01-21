mod timeseries;
mod load_vcd;

use crate::timeseries::*;
use crate::load_vcd::*;

use crossterm::ExecutableCommand;
use ratatui::style::{Style, Stylize, Color};
use ratatui::text::{Line, Span};

use std::env;

fn format_time_series(name: String, timeline: &ValueChangeStream, t_from: u64, t_to: u64) -> Line {
    let mut current_t = 0;
    let mut current_v = Value::Bits(Bits::Z);

    let change_from = timeline.index_at(t_from).unwrap_or(0);
    let change_to   = timeline.index_at(t_to).map(|i| i+1 ).unwrap_or(0);

    let mut spans = Vec::new();
    spans.push(Span::styled(name, Style::new().fg(Color::LightGreen).bg(Color::Black)));

    let style_bit = Style::new().fg(Color::LightGreen).bg(Color::Black);
    let style_var = Style::new().fg(Color::Black).bg(Color::LightGreen);
    let style_bad = Style::new().fg(Color::Black).bg(Color::LightRed);

    let mut currently_bad = false;

    for i in change_from..change_to {
        let change = &timeline.history[i];
        // print the current value
        for t in current_t..change.time {
            let (mut txt, sty) = match current_v {
                Value::Bits(bits) => {
                    match bits {
                        Bits::B(x) => {
                            currently_bad = false;
                            if x {
                                ("████".to_string(), style_bit)
                            } else {
                                ("▁▁▁▁".to_string(), style_bit)
                            }
                        }
                        Bits::V(x) => {
                            currently_bad = false;
                            (format!("{:<4x}", x.value), style_var)
                        }
                        Bits::X => {
                            currently_bad = true;
                            (" X  ".to_string(), style_bad)
                        }
                        Bits::Z => {
                            currently_bad = true;
                            (" Z  ".to_string(), style_bad)
                        }
                    }
                }
                Value::Real(x) => {
                    currently_bad = false;
                    (format!("{:<4}", x), style_var)
                }
                Value::String(ref x) => {
                    currently_bad = false;
                    (format!("{:<4}", x), style_var)
                }
            };
            if t+1 == change.time {
                txt.pop();
                txt.pop();
            }
            spans.push(Span::styled(txt, sty));
        }
        if i != 0 {
            match change.new_value {
                Value::Bits(bits) => {
                    match bits {
                        Bits::B(x) => {
                            if x {
                                spans.push(Span::styled("▁".to_string(), style_bit));
                            } else {
                                spans.push(Span::styled("▁".to_string(), style_bit));
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
        }
        current_v = change.new_value.clone();
        current_t = change.time;
    }

    for _ in current_t..t_to {
        let (txt, sty) = match current_v {
            Value::Bits(bits) => {
                match bits {
                    Bits::B(x) => {
                        if x {
                            ("████".to_string(), style_bit)
                        } else {
                            ("▁▁▁▁".to_string(), style_bit)
                        }
                    }
                    Bits::V(x) => {
                        (format!("{:<4x}", x.value), style_var)
                    }
                    Bits::X => {
                        (" X  ".to_string(), style_bad)
                    }
                    Bits::Z => {
                        (" Z  ".to_string(), style_bad)
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

// fn print_values(ts: &TimeSeries, s: &Scope, t_from: u64, t_to: u64) {
//     println!("scope {}", s.name);
//     for item in s.items.iter() {
//         match item {
//             ScopeItem::Value(v) => {
//                 println!("{:20}:{}", v.name, format_time_series(&ts.values[v.index], t_from, t_to));
//             }
//             ScopeItem::Scope(_) => {
//                 // do nothing
//             }
//         }
//     }
//     for item in s.items.iter() {
//         match item {
//             ScopeItem::Value(_) => {
//                 // do nothing
//             }
//             ScopeItem::Scope(subscope) => {
//                 print_values(ts, subscope, t_from, t_to);
//             }
//         }
//     }
// }

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: ./tuiwave [filename.vcd]");
        return;
    }

    let f = std::fs::File::open(&args[1]).unwrap();
    let ts = load_vcd(std::io::BufReader::new(f)).unwrap();

    let mut t_last = 0;
    for vs in ts.values.iter() {
        for change in vs.history.iter() {
            if t_last < change.time {
                t_last = change.time;
            }
        }
    }
    // print_values(&ts, &ts.scope, 0, t_last + 1);

    std::io::stdout().execute(crossterm::terminal::EnterAlternateScreen).unwrap();
    crossterm::terminal::enable_raw_mode().unwrap();
    let mut terminal = ratatui::terminal::Terminal::new(
        ratatui::backend::CrosstermBackend::new(std::io::stdout())).unwrap();
    terminal.clear().unwrap();

    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(
                ratatui::widgets::Paragraph::new(show_values(&ts, &ts.scope, 0, t_last+1)),
                area,
            );
        }).unwrap();

        if crossterm::event::poll(std::time::Duration::from_millis(16)).unwrap() {
            if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap() {
                if key.kind == crossterm::event::KeyEventKind::Press
                    && key.code == crossterm::event::KeyCode::Char('q')
                {
                    break;
                }
            }
        }
    }

    std::io::stdout().execute(crossterm::terminal::LeaveAlternateScreen).unwrap();
    crossterm::terminal::disable_raw_mode().unwrap();

    return ;
}
