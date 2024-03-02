use crate::timeseries::*;
use ratatui::layout::Rect;

use crossterm::event::{
    KeyCode, KeyModifiers, KeyEventState
};

pub struct TuiWave {
    pub ts: TimeSeries,
    pub t_from: u64,
    pub t_to:   u64,
    pub t_last: u64,
    pub width: u64,
    pub line_from: usize,
    pub line_focused: usize,
    pub current_drawable_lines: usize,
    pub current_stream_width: u64,
    pub current_sidebar_width_percent: u16,
    pub current_signame_width_percent: u16,
    pub should_quit: bool,
}

impl TuiWave {
    pub fn new(ts: TimeSeries) -> Self {
        let t_last = ts.values.iter().map(|v| v.last_change_time()).max().unwrap_or(0);
        Self{
            ts,
            t_from: 0,
            t_to: t_last+1,
            t_last,
            width: 4,
            line_from: 0,
            line_focused: 0,
            current_drawable_lines: 0,
            current_stream_width: t_last + 1,
            current_sidebar_width_percent: 15,
            current_signame_width_percent: 15,
            should_quit: false
        }
    }

    pub fn setup_with_terminal_size(&mut self, termsize: Rect) {
        let n_lines = termsize.height as usize / 2;
        let n_lines = if termsize.height % 2 == 1 { n_lines } else { n_lines - 1 };
        self.current_drawable_lines = n_lines;

        let main_pane = termsize.width * self.current_sidebar_width_percent / 100;
        self.current_stream_width = (main_pane * (100 - self.current_signame_width_percent) / 100) as u64;
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

            if (self.current_drawable_lines + self.line_from).saturating_sub(1) < self.line_focused {
                self.line_from = self.line_focused - self.current_drawable_lines + 1;
            }
        } else if key == KeyCode::Char('k') || key == KeyCode::Up {
            self.line_focused = self.line_focused.saturating_sub(1);
            if self.line_focused < self.line_from {
                self.line_from = self.line_focused;
            }
        } else if key == KeyCode::Char('-') {
            self.width = self.width.saturating_sub(1).max(2);
        } else if key == KeyCode::Char('+') {
            self.width = self.width.saturating_add(1).max(2);
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

    pub fn resize(&mut self, _w: u16, h: u16) {
        let n_lines = h as usize / 2;
        let n_lines = if h % 2 == 1 { n_lines } else { n_lines - 1 };

        if self.line_focused < self.line_from {
            self.line_from = self.line_focused;
        }
        if (n_lines + self.line_from).saturating_sub(1) < self.line_focused {
            self.line_from = self.line_focused - n_lines + 1;
        }
        self.current_drawable_lines = n_lines;
    }
}
