mod timeseries;
mod load_vcd;

use crate::timeseries::*;
use crate::load_vcd::*;

use crossterm::ExecutableCommand;
use ratatui::style::Stylize;

use std::env;

fn format_time_series(timeline: &ValueChangeStream, t_from: u64, t_to: u64) -> String {
    let mut s = String::new();
    let mut t = 0;
    let mut v = Value::Bits(Bits::Z);

    let change_from = timeline.index_at(t_from).unwrap_or(0);
    let change_to   = timeline.index_at(t_to).map(|i| i+1 ).unwrap_or(0);

    for i in change_from..change_to {
        let change = &timeline.history[i];
        // print the current value
        for _ in t..change.time {
            match v {
                Value::Bits(bits) => {
                    match bits {
                        Bits::B(x) => {
                            if x {
                                s += "▁▁▁▁";
                            } else {
                                s += "████";
                            }
                        }
                        Bits::V(x) => {
                            if x.width == 1 {
                                s += &format!("{:<4x}", x.value);
                            } else {
                                s += &format!("{:<4x}", x.value);
                            }
                        }
                        Bits::X => {
                            s += "X   ";
                        }
                        Bits::Z => {
                            s += "Z   ";
                        }
                    }
                }
                Value::Real(x) => {
                    s += &format!("{:4}", x);
                }
                Value::String(ref x) => {
                    s += &format!("{:4}", x);
                }
            }
        }
        if i != 0 {
            s.pop();
            s.pop();
            s += if let Value::Bits(Bits::B(x)) = change.new_value {
                    if x { "▁" } else { "▁" }
                } else {
                    ""
                };
        }
        v = change.new_value.clone();
        t = change.time;
    }
    for _ in t..t_to {
        match v {
            Value::Bits(bits) => {
                match bits {
                    Bits::B(x) => {
                        if x {
                            s += "▁▁▁▁";
                        } else {
                            s += "████";
                        }
                    }
                    Bits::V(x) => {
                        if x.width == 1 {
                            s += &format!("{:<4x}", x.value);
                        } else {
                            s += &format!("{:<4x}", x.value);
                        }
                    }
                    Bits::X => {
                        s += "X   ";
                    }
                    Bits::Z => {
                        s += "Z   ";
                    }
                }
            }
            Value::Real(x) => {
                s += &format!("{:4}", x);
            }
            Value::String(ref x) => {
                s += &format!("{:4}", x);
            }
        }
    }
    s
}

fn print_values(ts: &TimeSeries, s: &Scope, t_from: u64, t_to: u64) {
    println!("scope {}", s.name);
    for item in s.items.iter() {
        match item {
            ScopeItem::Value(v) => {
                println!("{:20}:{}", v.name, format_time_series(&ts.values[v.index], t_from, t_to));
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
                print_values(ts, subscope, t_from, t_to);
            }
        }
    }
}

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
    print_values(&ts, &ts.scope, 0, t_last + 1);

    std::io::stdout().execute(crossterm::terminal::EnterAlternateScreen).unwrap();
    crossterm::terminal::enable_raw_mode().unwrap();
    let mut terminal = ratatui::terminal::Terminal::new(
        ratatui::backend::CrosstermBackend::new(std::io::stdout())).unwrap();
    terminal.clear().unwrap();

    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(
                ratatui::widgets::Paragraph::new("Hello Ratatui! (press 'q' to quit)")
                    .white()
                    .on_blue(),
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
