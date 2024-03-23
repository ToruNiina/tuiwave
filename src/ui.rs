use crate::timeseries::*;
use crate::app;

use ratatui::symbols;
use ratatui::style::{Style, Stylize, Color};
use ratatui::text::{Line, Span, Text};
use ratatui::terminal::Frame;
use ratatui::layout::{Layout, Constraint, Direction, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct StyledString {
    string: String,
    style: Style,
}

impl StyledString {
    fn styled(string: String, style: Style) -> Self {
        Self {string: string.to_string(), style}
    }
    fn to_span(&self) -> Span {
        Span::styled(self.string.clone(), self.style)
    }
}

fn format_time_series(timeline: &ValueChangeStream, t_from: u64, t_to: u64, width: u64) -> Vec<StyledString> {
    if let ValueChangeStream::Bits(ts) = timeline {
        return format_time_series_bits(ts, t_from, t_to, width);
    } else {
        panic!("type is unknown -> {:?}", timeline);
    }
}

fn format_time_series_bits(timeline: &ValueChangeStreamImpl<Bits>, t_from: u64, t_to: u64, width: u64) -> Vec<StyledString> {
    let mut current_t = t_from;
    let mut current_v = Bits::Z;

    if let Some(before_start) = timeline.change_before(t_from) {
        current_v = timeline.stream[before_start].new_value.clone();
    }
    let change_from = timeline.change_after(t_from);
    let change_to   = timeline.change_after(t_to  );

    // eprintln!("format_time_series({}, t_from={}, t_to={}): change_from = {:?}, to = {:?}", name, t_from, t_to, change_from, change_to);

    let mut spans = Vec::new();

    let style_bit = Style::new().fg(Color::LightGreen).bg(Color::Black);
    let style_var = Style::new().fg(Color::Black).bg(Color::LightGreen);
    let style_bad = Style::new().fg(Color::Black).bg(Color::LightRed);

    if let Some(change_from) = change_from {
        let change_to = change_to.unwrap_or(timeline.stream.len());

        for i in change_from..change_to {
            let mut currently_bad = false; // Z or X

            let change = &timeline.stream[i];

            let dt = (change.time - current_t).max(1);
            let w  = (width * dt - 2) as usize;

            let (mut txt, sty) = match current_v {
                Bits::B(x) => {
                    if x {
                        ("▇".repeat(w), style_bit)
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
            };

            assert!(txt.chars().count() >= w);
            if txt.chars().count() > w {
                txt = txt.chars().take(w).collect();
            }
            spans.push(StyledString::styled(txt, sty));

            // total_width += 2;
            match change.new_value {
                Bits::B(x) => {
                    if x {
                        spans.push(StyledString::styled("▇".to_string(), style_bit));
                    } else {
                        spans.push(StyledString::styled("▁".to_string(), style_bit));
                    }
                }
                Bits::V(_) => {
                    spans.push(StyledString::styled("".to_string(),
                        Style::new().fg(Color::LightGreen).bg(Color::Black)
                    ));
                }
                Bits::X => {
                    if currently_bad {
                        spans.push(StyledString::styled("".to_string(),
                            Style::new().fg(Color::LightRed).bg(Color::Black)
                        ));
                    } else {
                        spans.push(StyledString::styled("".to_string(),
                            Style::new().fg(Color::LightGreen).bg(Color::Black)
                        ));
                        spans.push(StyledString::styled("".to_string(),
                            Style::new().fg(Color::LightRed).bg(Color::Black)
                        ));
                    }
                }
                Bits::Z => {
                    if currently_bad {
                        spans.push(StyledString::styled("".to_string(),
                            Style::new().fg(Color::LightRed).bg(Color::Black)
                        ));
                    } else {
                        spans.push(StyledString::styled("".to_string(),
                            Style::new().fg(Color::LightGreen).bg(Color::Black)
                        ));
                        spans.push(StyledString::styled("".to_string(),
                            Style::new().fg(Color::LightRed).bg(Color::Black)
                        ));
                    }
                }
            };
            current_v = change.new_value.clone();
            current_t = change.time;
        }
    }

    if current_t < t_to {
        let dt = (t_to - current_t).max(1);
        let w = (width * dt) as usize;

        let span = match current_v {
            Bits::B(x) => {
                if x {
                    StyledString::styled("▇".repeat(w), style_bit)
                } else {
                    StyledString::styled("▁".repeat(w), style_bit)
                }
            }
            Bits::V(x) => {
                StyledString::styled(format!("{:<width$x}", x.value, width=w), style_var)
            }
            Bits::X => {
                StyledString::styled(format!("{:<width$}", "X", width=w), style_bad)
            }
            Bits::Z => {
                StyledString::styled(format!("{:<width$}", "Z", width=w), style_bad)
            }
        };
        spans.push(span);
    }
    spans
}

pub fn format_values(app: & app::TuiWave, values: &[((String, String), usize)])
    -> Vec<((StyledString, StyledString), Vec<StyledString>)>
{
    let mut lines = Vec::new();
    for ((path, name), idx) in values.iter() {
        let line = format_time_series(
            &app.ts.values[*idx],
            app.t_from,
            app.t_to.min(app.t_last+1),
            app.layout.timedelta_width);

        let path = StyledString::styled(path.clone(), Style::default().fg(Color::DarkGray));
        let name = StyledString::styled(name.clone(), Style::default().bold());

        lines.push( ( (path, name), line) );
    }
    lines
}

fn draw_waveform(app: &app::TuiWave, frame: &mut Frame, chunk: &Rect) {

    let lines = &app.cache.signal_timelines;

    // the first line has all (including top and bottom) borders so takes 3 lines.
    let mut constraints = vec![Constraint::Length(3)];
    // other lines does not have top border. takes 2 lines.
    constraints.extend(vec![Constraint::Length(2); lines.len()-1]);

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

        let ((path, name), line) = &lines[idx];

        let sublayout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(Constraint::from_percentages([
                app.layout.signame_width_percent,
                100 - app.layout.signame_width_percent
            ]))
            .split(layout[idx]);

        let relative_focus = app.focus_signal - app.line_from;
        let is_focused = (idx == relative_focus) && app.focus == app::Focus::Signal;
        let next_focused = !is_last && (idx+1) == relative_focus && app.focus == app::Focus::Signal;

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                path.to_span(),
                name.to_span(),
            ])).block(Block::new()
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

        let spans = line.iter()
            .map(|s| s.to_span())
            .collect::<Vec<_>>();

        frame.render_widget(
            Paragraph::new(Line::from(spans))
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

fn draw_sidebar(app: &app::TuiWave, frame: &mut Frame, chunk: &Rect) {

    let values = &app.cache.selected_values;
    let name_size = 3 + values.len() * 2 + 1;
    let names = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(name_size as u16),
            Constraint::Fill(1)
        ])
        .split(*chunk);

    let tree = &app.cache.scope_tree_lines;

    let mut lines = Vec::new();
    for (i, s) in tree.iter().enumerate() {
        let sty = if i == app.focus_tree {
            if app.focus == app::Focus::Tree {
                Style::new().underlined().bold()
            } else {
                Style::new().underlined()
            }
        } else {
            Style::new()
        };
        lines.push(Line::styled(s, sty));
    }

    frame.render_widget(
        Paragraph::new(
            Text::from(lines)
        ).block(
            Block::new()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(Color::DarkGray))
            .border_set(if app.focus == app::Focus::Tree {
                symbols::border::THICK
            } else {
                symbols::border::PLAIN
            })
        ),
        names[0]);
}

fn make_tick(app: &app::TuiWave, tick: &str) -> String {
    "─".repeat((app.layout.timedelta_width-1) as usize) + tick
}

fn make_ruler(app: &app::TuiWave) -> (StyledString, StyledString) {

    let t_from = app.t_from as usize;
    let t_to   = app.t_to   as usize;
    let t_range = t_to - t_from;
    let t_width = app.layout.timedelta_width as usize;

    assert!(t_width >= 2);

    let tick       = make_tick(app, "┬");
    let first_tick = tick.repeat(9 - (t_from % 10)) + &make_tick(app, "╥");
    let tick_x10   = tick.repeat(9)                 + &make_tick(app, "╥");
    let ruler      = first_tick + &tick_x10.repeat(t_range / 10 + 1);

    let mut labels = {
        let w = t_width * (10 - t_from%10);
        let label = format!("{:>width$}", (t_from/10 + 1) * 10, width=w);
        label[0..w].to_string()
    };
    for i in 1..(t_range/10 + 1) {
        let t = t_from / 10 * 10 + ((i+1) * 10);
        let w = t_width * 10;
        let label = format!("{:>width$}", t, width=w);
        labels += &label[0..w];
    }

    (
        StyledString::styled(labels, Style::default()),
        StyledString::styled(ruler, Style::default())
    )
}

fn draw_ruler(app: &app::TuiWave, frame: &mut Frame, chunk: &Rect) {

    let sublayout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_percentages([
            app.layout.signame_width_percent,
            100 - app.layout.signame_width_percent
        ]))
        .split(*chunk);

    let ruler_border = symbols::border::Set {
        top_left: "┬",
        bottom_left: "┴",
        .. symbols::border::PLAIN
    };

    frame.render_widget(
        Paragraph::new(
            Text::from(format!("time [{} {}]", app.ts.time_scale.0, app.ts.time_scale.1))
        ).block(
            Block::new()
            .borders(Borders::TOP | Borders::LEFT)
            .border_style(Style::new().fg(Color::DarkGray))
            .border_set(symbols::border::PLAIN)
        ),
        sublayout[0]);

    let (labels, ruler) = make_ruler(app);

    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(labels.string, labels.style),
            Line::styled(ruler.string, ruler.style),
        ]).block(
            Block::new()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::new().fg(Color::DarkGray))
            .border_set(ruler_border)
        ),
        sublayout[1]);
}

pub fn draw_ui(app: &app::TuiWave, frame: &mut Frame) {
    // add side bar showing a list of signals
    let root = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_percentages([
            app.layout.sidebar_width_percent,
            100 - app.layout.sidebar_width_percent
        ]))
        .split(frame.size());

    // add ruler on top of waveform

    let waveform = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .split(root[1]);

    draw_sidebar(app, frame, &root[0]);
    draw_ruler(app, frame, &waveform[0]);
    draw_waveform(app, frame, &waveform[1]);
}
