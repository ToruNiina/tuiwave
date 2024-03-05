use crate::timeseries::*;
use crate::ui;

use ratatui::layout::Rect;
use ratatui::style::Style;

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

impl Layout {
    fn resize(&mut self, w: u16, h: u16) {
        self.current_width = w;
        self.current_height = h;

        let n_lines = self.current_height as usize / 2;
        let n_lines = if self.current_height % 2 == 1 { n_lines } else { n_lines - 1 };
        self.drawable_lines = n_lines;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Signal,
    Tree,
}

pub struct TuiWave {
    pub ts: TimeSeries,
    pub cache: UICache,
    pub t_from: u64,
    pub t_to:   u64,
    pub t_last: u64,
    pub line_from: usize,
    pub layout: Layout,
    pub should_quit: bool,
    pub window_change_mode: bool,

    pub focus: Focus,
    pub focus_signal: usize,
    pub focus_tree: usize,
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
        let cache = UICache::new(&ts);

        Self{
            ts,
            cache,
            t_from: 0,
            t_to: t_last+1,
            t_last,
            line_from: 0,
            layout,
            should_quit: false,
            window_change_mode: false,
            focus: Focus::Signal,
            focus_signal: 0,
            focus_tree: 0,
        }
    }

    // call it after resizing the window, or changed the rayout parameters
    fn setup_drawable_time_range(&mut self) {
        let main_pane = self.layout.current_width * (100 - self.layout.sidebar_width_percent) / 100;
        self.layout.stream_width = (main_pane * (100 - self.layout.signame_width_percent) / 100) as u64;
        let time_range = self.layout.stream_width / self.layout.timedelta_width;
        self.t_to = (self.t_from + time_range).min(self.t_last+1);
    }

    pub fn setup_with_terminal_size(&mut self, termsize: Rect) {
        self.layout.resize(termsize.width, termsize.height);
        self.setup_drawable_time_range();
        self.render_waveform();
    }

    pub fn key_press(&mut self, key: KeyCode, modifiers: KeyModifiers, _state: KeyEventState) {
        if key == KeyCode::Char('q') {
            self.should_quit = true;
        } else if key == KeyCode::Char('l') || key == KeyCode::Right {
            if self.window_change_mode {
                self.focus = Focus::Signal;
                self.window_change_mode = false;
            } else if self.focus == Focus::Signal {
                self.t_from = self.t_from.saturating_add(1);
                self.t_to   = self.t_to  .saturating_add(1);
                self.render_waveform();
            }
        } else if key == KeyCode::Char('h') || key == KeyCode::Left {
            if self.window_change_mode {
                self.focus = Focus::Tree;
                self.window_change_mode = false;
            } else if self.focus == Focus::Signal {
                if self.t_from != 0 {
                    self.t_from = self.t_from.saturating_sub(1);
                    self.t_to   = self.t_to  .saturating_sub(1);
                    self.render_waveform();
                }
            }
        } else if key == KeyCode::Char('j') || key == KeyCode::Down {
            if self.focus == Focus::Signal {
                self.focus_signal =
                    (self.focus_signal + 1).min(self.ts.values.len().saturating_sub(1));

                if (self.layout.drawable_lines + self.line_from).saturating_sub(1) < self.focus_signal {
                    self.line_from = self.focus_signal - self.layout.drawable_lines + 1;
                }
            } else {
                self.focus_tree = (self.focus_tree + 1).min(self.cache.scope_tree_lines.len().saturating_sub(1));
            }
        } else if key == KeyCode::Char('k') || key == KeyCode::Up {
            if self.focus == Focus::Signal {
                self.focus_signal = self.focus_signal.saturating_sub(1);
                if self.focus_signal < self.line_from {
                    self.line_from = self.focus_signal;
                }
            } else {
                self.focus_tree = self.focus_tree.saturating_sub(1)
            }
        } else if key == KeyCode::Char('-') {
            self.layout.timedelta_width = self.layout.timedelta_width.saturating_sub(1).max(2);
            self.setup_drawable_time_range();
            self.render_waveform();
        } else if key == KeyCode::Char('+') {
            self.layout.timedelta_width = self.layout.timedelta_width.saturating_add(1).max(2);
            self.setup_drawable_time_range();
            self.render_waveform();
        } else if key == KeyCode::Char('0') {
            let dt = self.t_to.saturating_sub(self.t_from);
            self.t_to   = dt;
            self.t_from = 0;
            self.render_waveform();
        } else if key == KeyCode::Char('$') {
            let dt = self.t_to.saturating_sub(self.t_from);
            self.t_to   = self.t_last;
            self.t_from = self.t_last.saturating_sub(dt);
            self.render_waveform();
        } else if modifiers == KeyModifiers::CONTROL && key == KeyCode::Char('w') {
            self.window_change_mode = true;
        } else if key == KeyCode::Enter {
            if self.focus == Focus::Tree {
                self.flip_scope_tree();
                self.cache.update_selection(&self.ts);
                self.render_waveform();
            }
        }
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        self.layout.resize(w, h);

        let n_lines = h as usize / 2;
        let n_lines = if h % 2 == 1 { n_lines } else { n_lines - 1 };

        if self.focus_signal < self.line_from {
            self.line_from = self.focus_signal;
        }
        if (n_lines + self.line_from).saturating_sub(1) < self.focus_signal {
            self.line_from = self.focus_signal - n_lines + 1;
        }
    }

    fn flip_scope_tree_impl(node: &mut Scope, i: &mut usize, flipped: usize) -> bool {
        if *i == flipped {
            node.open = !node.open;
            return true;
        }
        *i += 1;

        if !node.open {
            return false;
        }

        for item in node.items.iter_mut() {
            if let ScopeItem::Value(v) = item {
                if *i == flipped {
                    v.render = !v.render;
                    return true;
                }
                *i += 1;
            }
        }
        for item in node.items.iter_mut() {
            if let ScopeItem::Scope(s) = item {
                let done = Self::flip_scope_tree_impl(s, i, flipped);
                if done {
                    return true;
                }
            }
        }
        return false;
    }
    fn flip_scope_tree(&mut self) {
        let mut idx = 0;
        let done =  Self::flip_scope_tree_impl(&mut self.ts.scope, &mut idx, self.focus_tree);
        assert!(done);
    }

    fn render_waveform(&mut self) {
        let values = &self.cache.selected_values;
        let line_to = (self.line_from + self.layout.drawable_lines-1).min(values.len());
        self.cache.signal_timelines = ui::format_values(&self, &values[self.line_from..line_to]);
    }
}

pub struct UICache {
    pub selected_values: Vec<((String, String), usize)>,
    pub scope_tree_lines: Vec<String>,
    pub signal_timelines: Vec<((String, String), Vec<(String, Style)>)>,
}

impl UICache {
    pub fn new(ts: &TimeSeries) -> Self {
        Self {
            selected_values: Self::list_values(&ts.scope),
            scope_tree_lines: Self::draw_scope_tree(&ts.scope),
            signal_timelines: Vec::new(),
        }
    }

    pub fn update_selection(&mut self, ts: &TimeSeries) {
        self.selected_values = Self::list_values(&ts.scope);
        self.scope_tree_lines = Self::draw_scope_tree(&ts.scope);
    }

    fn list_values_impl(s: &Scope, path: &str, vs: &mut Vec<((String, String), usize)>) {
        for item in s.items.iter() {
            if let ScopeItem::Value(v) = item {
                if !v.should_be_rendered() {
                    continue;
                }
                let mut path_to_item = path.to_string();
                path_to_item += ".";
                vs.push(((path_to_item, v.name.clone()), v.index));
            }
        }
        for item in s.items.iter() {
            if let ScopeItem::Scope(subscope) = item {
                if !subscope.should_be_rendered() {
                    continue;
                }
                let mut path_to_item = path.to_string();
                path_to_item += ".";
                path_to_item += &subscope.name[0..1];

                Self::list_values_impl(subscope, &path_to_item, vs);
            }
        }
    }

    fn list_values(root: &Scope) -> Vec<((String, String), usize)> {
        let mut vs = Vec::new();
        Self::list_values_impl(&root, &root.name[0..1], &mut vs);
        vs
    }

    fn draw_scope_tree_impl(s: &Scope, lines: &mut Vec<String>, indent: String) {
        if ! s.open {
            return ;
        }

        let n_values: usize = s.items.iter().map(|x| {
            if let ScopeItem::Value(_) = x { 1 } else { 0 }
        }).sum();

        let n_scopes: usize = s.items.iter().map(|x| {
            if let ScopeItem::Scope(_) = x { 1 } else { 0 }
        }).sum();

        let mut c_values = 0;
        for item in s.items.iter() {
            let is_last = (n_scopes == 0) && (c_values + 1) == n_values;
            if let ScopeItem::Value(v) = item {
                let cbox = if v.should_be_rendered() { "☑"  } else { "☐"  };
                let branch = if is_last { "└" } else { "├" };
                lines.push(format!("{}{}╴{} {}", indent, branch, cbox, v.name));
                c_values += 1;
            }
        }

        let mut c_scopes = 0;
        for item in s.items.iter() {
            let is_last = (c_scopes + 1) == n_scopes;
            if let ScopeItem::Scope(subscope) = item {
                let branch = if is_last { "└" } else { "├" };
                let next_indent = indent.clone() + (if is_last { "  " } else { "│ " });
                let open_icon = if subscope.open { "▼" } else { "▶" };
                lines.push(format!("{}{}╴{} {}", indent, branch, open_icon, subscope.name));

                Self::draw_scope_tree_impl(subscope, lines, next_indent);
                c_scopes += 1;
            }
        }
    }
    fn draw_scope_tree(root: &Scope) -> Vec<String> {
        let mut tree = vec![root.name.clone()];
        Self::draw_scope_tree_impl(&root, &mut tree, "".to_string());
        tree
    }
}
