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

impl Layout {
    fn resize(&mut self, w: u16, h: u16) {
        self.current_width = w;
        self.current_height = h;

        let n_lines = self.current_height as usize / 2;
        let n_lines = if self.current_height % 2 == 1 { n_lines } else { n_lines - 1 };
        self.drawable_lines = n_lines;
    }
}

pub struct TuiWave {
    pub ts: TimeSeries,
    pub cache: UICache,
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
        let cache = UICache::new(&ts);

        Self{
            ts,
            cache,
            t_from: 0,
            t_to: t_last+1,
            t_last,
            line_from: 0,
            line_focused: 0,
            layout,
            should_quit: false
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
            self.setup_drawable_time_range();
        } else if key == KeyCode::Char('+') {
            self.layout.timedelta_width = self.layout.timedelta_width.saturating_add(1).max(2);
            self.setup_drawable_time_range();
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
        self.layout.resize(w, h);

        let n_lines = h as usize / 2;
        let n_lines = if h % 2 == 1 { n_lines } else { n_lines - 1 };

        if self.line_focused < self.line_from {
            self.line_from = self.line_focused;
        }
        if (n_lines + self.line_from).saturating_sub(1) < self.line_focused {
            self.line_from = self.line_focused - n_lines + 1;
        }
    }

}

pub struct UICache {
    pub selected_values: Vec<(String, usize)>,
    pub scope_tree_lines: Vec<String>,
}

impl UICache {
    pub fn new(ts: &TimeSeries) -> Self {
        Self {
            selected_values: Self::list_values(&ts.scope),
            scope_tree_lines: Self::draw_scope_tree(&ts.scope),
        }
    }

    pub fn update(&mut self, ts: &TimeSeries) {
        selected_values = Self::list_values(&ts.scope);
        scope_tree_lines = Self::draw_scope_tree(&ts.scope);
    }

    fn list_values_impl(s: &Scope, path: &String, vs: &mut Vec<(String, usize)>) {
        for item in s.items.iter() {
            if let ScopeItem::Value(v) = item {
                if !v.should_be_rendered() {
                    continue;
                }
                let mut path_to_item = path.clone();
                path_to_item += ".";
                path_to_item += &v.name;
                vs.push((path_to_item, v.index));
            }
        }
        for item in s.items.iter() {
            if let ScopeItem::Scope(subscope) = item {
                if !subscope.should_be_rendered() {
                    continue;
                }
                let mut path_to_item = path.clone();
                path_to_item += ".";
                path_to_item += &subscope.name;

                Self::list_values_impl(subscope, &path_to_item, vs);
            }
        }
    }

    fn list_values(root: &Scope) -> Vec<(String, usize)> {
        let mut vs = Vec::new();
        Self::list_values_impl(&root, &root.name, &mut vs);
        vs
    }

    fn draw_scope_tree_impl(s: &Scope, lines: &mut Vec<String>, indent: String) {

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
                lines.push(format!("{}{}╴{}", indent, branch, subscope.name));

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


