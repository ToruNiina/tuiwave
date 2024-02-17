use crate::timeseries::*;

use ratatui::style::{Style, Color};
use ratatui::text::{Line, Span};

pub struct TuiWave {
    pub ts: TimeSeries,
    pub t_from: u64,
    pub t_to:   u64,
    pub t_last: u64,
    pub width: u64,
    pub line_from: usize,
    pub line_focused: usize,
    pub current_drawable_lines: usize,
    pub should_quit: bool,
}

impl TuiWave {
    pub fn new(ts: TimeSeries) -> Self {
        let mut t_last = 0;
        for vs in ts.values.iter() {
            for change in vs.history.iter() {
                if t_last < change.time {
                    t_last = change.time;
                }
            }
        }
        Self{ ts, t_from: 0, t_to: t_last+1, t_last, width: 4, line_from: 0, line_focused: 0, current_drawable_lines: 0, should_quit: false}
    }
}

fn format_time_series(timeline: &ValueChangeStream, t_from: u64, t_to: u64, width: u64)
    -> Line
{
    let mut current_t = t_from;
    let mut current_v = Value::Bits(Bits::Z);

    if let Some(before_start) = timeline.change_before(t_from) {
        current_v = timeline.history[before_start].new_value.clone();
    }
    let change_from = timeline.change_after(t_from);
    let change_to   = timeline.change_after(t_to  );

    // eprintln!("format_time_series({}, t_from={}, t_to={}): change_from = {:?}, to = {:?}", name, t_from, t_to, change_from, change_to);

    let mut spans = Vec::new();

    let style_bit = Style::new().fg(Color::LightGreen).bg(Color::Black);
    let style_var = Style::new().fg(Color::Black).bg(Color::LightGreen);
    let style_bad = Style::new().fg(Color::Black).bg(Color::LightRed);

    if let Some(change_from) = change_from {
        let change_to = change_to.unwrap_or(timeline.history.len());

        for i in change_from..change_to {
            let mut currently_bad = false; // Z or X

            let change = &timeline.history[i];

            let dt = (change.time - current_t).max(1);
            let w  = (width * dt - 2) as usize;

            let (mut txt, sty) = match current_v {
                Value::Bits(bits) => {
                    match bits {
                        Bits::B(x) => {
                            if x {
                                ("▔".repeat(w), style_bit)
                            } else {
                                ("▁".repeat(w), style_bit)
                            }
                        }
                        Bits::V(x) => {
                            (format!("{:<width$x}", x.value, width = w), style_var)
                        }
                        Bits::X => {
                            currently_bad = true;
                            (format!("{:<width$}", "X", width = w), style_bad)
                        }
                        Bits::Z => {
                            currently_bad = true;
                            (format!("{:<width$}", "Z", width = w), style_bad)
                        }
                    }
                }
                Value::Real(x) => {
                    (format!("{:<width$}", x, width = w), style_var)
                }
                Value::String(ref x) => {
                    (format!("{:<width$}", x, width = w), style_var)
                }
            };
            assert!(txt.chars().count() >= w);
            if txt.chars().count() > w {
                txt = txt.chars().take(w).collect();
            }
            spans.push(Span::styled(txt, sty));

            // total_width += 2;
            match change.new_value {
                Value::Bits(bits) => {
                    match bits {
                        Bits::B(x) => {
                            if x {
                                spans.push(Span::styled("／".to_string(), style_bit));
                            } else {
                                spans.push(Span::styled("＼".to_string(), style_bit));
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
        let dt = (t_to - current_t).max(1);
        let w = (width * dt) as usize;

        let (txt, sty) = match current_v {
            Value::Bits(bits) => {
                match bits {
                    Bits::B(x) => {
                        if x {
                            ("▔".repeat(w), style_bit)
                        } else {
                            ("▁".repeat(w), style_bit)
                        }
                    }
                    Bits::V(x) => {
                        (format!("{:<width$x}", x.value, width=w), style_var)
                    }
                    Bits::X => {
                        (format!("{:<width$}", "X", width=w), style_bad)
                    }
                    Bits::Z => {
                        (format!("{:<width$}", "Z", width=w), style_bad)
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

pub fn format_values<'a>(app: &'a TuiWave, values: &[(String, usize)])
    -> Vec<(String, Line<'a>)>
{
    let mut lines = Vec::new();
    for (path, idx) in values.iter() {
        let line = format_time_series(
            &app.ts.values[*idx],
            app.t_from,
            app.t_to.min(app.t_last+1),
            app.width);
        lines.push( (path.clone(), line) );
    }
    lines
}

pub fn list_values(app: &TuiWave, s: &Scope, path: &String) -> Vec<(String, usize)>
{
    let mut vs = Vec::new();
    for item in s.items.iter() {
        if let ScopeItem::Value(v) = item {
            let mut path_to_item = path.clone();
            path_to_item += ".";
            path_to_item += &v.name;
            vs.push((path_to_item, v.index));
        }
    }
    for item in s.items.iter() {
        if let ScopeItem::Scope(subscope) = item {
            let mut path_to_item = path.clone();
            path_to_item += ".";
            path_to_item += &subscope.name;

            let subvs = list_values(app, subscope, &path_to_item);
            vs.extend(subvs.into_iter());
        }
    }
    vs
}
