use crate::timeseries::*;
use crate::app;

use ratatui::symbols;
use ratatui::style::{Style, Color};
use ratatui::text::{Line, Span};
use ratatui::terminal::Frame;
use ratatui::layout::{Layout, Constraint, Direction, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};

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

pub fn format_values<'a>(app: &'a app::TuiWave, values: &[(String, usize)])
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

pub fn list_values(app: &app::TuiWave, s: &Scope, path: &String) -> Vec<(String, usize)>
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

fn draw_timeline(app: &app::TuiWave, frame: &mut Frame, chunk: &Rect) {

    let values = list_values(&app, &app.ts.scope, &app.ts.scope.name);
    let line_to = (app.line_from + app.current_drawable_lines-1).min(values.len());
    let lines = format_values(&app, &values[app.line_from..line_to]);

    // the first line has all (including top and bottom) borders so takes 3 lines.
    let mut constraints = vec![Constraint::Length(3)];
    // other lines does not have top border. takes 2 lines.
    constraints.extend(Constraint::from_lengths(std::iter::repeat(2).take(lines.len())));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(*chunk);

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

    for idx in 0..lines.len() {

        let is_first = idx == 0;
        let is_last = idx+1 == lines.len();

        let (path, line) = &lines[idx];

        let sublayout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(Constraint::from_percentages([15, 85]))
            .split(layout[idx]);

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
    }
}

pub fn draw_ui(app: &app::TuiWave, frame: &mut Frame) {

    // add side bar showing a list of signals
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_percentages([15, 85]))
        .split(frame.size());

    let values = list_values(&app, &app.ts.scope, &app.ts.scope.name);

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

    draw_timeline(app, frame, &layout[1]);
}
