use crate::timeseries::*;

use ratatui::style::{Style, Color};
use ratatui::text::{Line, Span};

pub struct TuiWave {
    pub ts: TimeSeries,
    pub t_from: u64,
    pub t_to:   u64,
    pub t_last: u64,
    pub resolution: u64,
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
        Self{ ts, t_from: 0, t_to: t_last+1, t_last, resolution: 1, should_quit: false}
    }
}

fn format_time_series(name: String, timeline: &ValueChangeStream, t_from: u64, t_to: u64, res: u64) -> Line {

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

            let dt = (change.time - current_t) / res;
            let dt = dt.max(1) as usize;
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
        let dt = (t_to - current_t) / res;
        let dt = dt.max(1) as usize;

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

pub fn show_values<'a>(app: &'a TuiWave, s: &'a Scope) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    for item in s.items.iter() {
        match item {
            ScopeItem::Value(v) => {
                lines.push(Line::from(""));
                lines.push(format_time_series(
                    format!("{:20}", v.name),
                    &app.ts.values[v.index],
                    app.t_from,
                    app.t_to.min(app.t_last+1),
                    app.resolution
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
                let ls = show_values(app, subscope);
                lines.extend(ls.into_iter());
            }
        }
    }
    lines
}

