use crate::timeseries::*;
use ratatui::layout::Rect;

use crossterm::event::{
    KeyCode, KeyModifiers, KeyEventState
};

pub struct Layout {
    pub drawable_lines: usize,
    pub stream_width: u64,
    pub sidebar_width_percent: u16,
    pub signame_width_percent: u16,
    pub timedelta_width: u64,

    pub current_width: u16,
    pub current_height: u16,
}

pub struct TuiWave {
    pub ts: TimeSeries,

    pub t_from: u64,
    pub t_to:   u64,
    pub t_last: u64,
    pub line_from: usize,
    pub line_focused: usize,
    pub layout: Layout,
    pub should_quit: bool,
}

impl TuiWave {
    pub fn new(ts: TimeSeries) -> Self {
        let t_last = ts.values.iter().map(|v| v.last_change_time()).max().unwrap_or(0);
        let layout = Layout{
            drawable_lines: 0,
            stream_width: t_last + 1,
            sidebar_width_percent: 15,
            signame_width_percent: 15,
            timedelta_width: 4,
            current_width: 0,
            current_height: 0,
        };
        Self{
            ts,
            t_from: 0,
            t_to: t_last+1,
            t_last,
            line_from: 0,
            line_focused: 0,
            layout,
            should_quit: false
        }
    }

    pub fn setup_with_terminal_size(&mut self, termsize: Rect) {
        self.layout.current_width  = termsize.width;
        self.layout.current_height = termsize.height;

        let n_lines = termsize.height as usize / 2;
        let n_lines = if termsize.height % 2 == 1 { n_lines } else { n_lines - 1 };
        self.layout.drawable_lines = n_lines;

        let main_pane = termsize.width * (100 - self.layout.sidebar_width_percent) / 100;
        self.layout.stream_width = (main_pane * (100 - self.layout.signame_width_percent) / 100) as u64;

        self.t_to = (self.layout.stream_width / self.layout.timedelta_width) + self.t_from;
        self.t_to = self.t_to.min(self.t_last+1)
    }

    pub fn key_press(&mut self, key: KeyCode, _modifiers: KeyModifiers, _state: KeyEventState) {
        if key == KeyCode::Char('q') {
            self.should_quit = true;
        } else if key == KeyCode::Char('l') || key == KeyCode::Right {
            self.t_from = self.t_from.saturating_add(1);
            self.t_to   = self.t_to  .saturating_add(1);
        } else if key == KeyCode::Char('h') || key == KeyCode::Left {
            if self.t_from != 0 {
                self.t_from = self.t_from.saturating_sub(1);
                self.t_to   = self.t_to  .saturating_sub(1);
            }
        } else if key == KeyCode::Char('j') || key == KeyCode::Down {
            self.line_focused = (self.line_focused + 1).min(self.ts.values.len().saturating_sub(1));

            if (self.layout.drawable_lines + self.line_from).saturating_sub(1) < self.line_focused {
                self.line_from = self.line_focused - self.layout.drawable_lines + 1;
            }
        } else if key == KeyCode::Char('k') || key == KeyCode::Up {
            self.line_focused = self.line_focused.saturating_sub(1);
            if self.line_focused < self.line_from {
                self.line_from = self.line_focused;
            }
        } else if key == KeyCode::Char('-') {
            self.layout.timedelta_width = self.layout.timedelta_width.saturating_sub(1).max(2);
        } else if key == KeyCode::Char('+') {
            self.layout.timedelta_width = self.layout.timedelta_width.saturating_add(1).max(2);
        } else if key == KeyCode::Char('0') {
            let dt = self.t_to.saturating_sub(self.t_from);
            self.t_to   = dt;
            self.t_from = 0;
        } else if key == KeyCode::Char('$') {
            let dt = self.t_to.saturating_sub(self.t_from);
            self.t_to   = self.t_last;
            self.t_from = self.t_last.saturating_sub(dt);
        }
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        self.layout.current_width  = w;
        self.layout.current_height = h;

        let n_lines = h as usize / 2;
        let n_lines = if h % 2 == 1 { n_lines } else { n_lines - 1 };

        if self.line_focused < self.line_from {
            self.line_from = self.line_focused;
        }
        if (n_lines + self.line_from).saturating_sub(1) < self.line_focused {
            self.line_from = self.line_focused - n_lines + 1;
        }
        self.layout.drawable_lines = n_lines;
    }
}
