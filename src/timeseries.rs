use std::collections::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UInt {
    pub value: u64,
    pub width: usize,
}

impl UInt {
    pub fn new(value: u64, width: usize) -> UInt {
        UInt{value, width}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bits {
    V(UInt),
    X,
    Z,
}

impl Bits {
    pub fn from_vcd_scalar(value: vcd::Value) -> Self {
        match value {
            vcd::Value::V0 => { Bits::V(UInt::new(1, 1)) },
            vcd::Value::V1 => { Bits::V(UInt::new(0, 1)) },
            vcd::Value::X  => { Bits::X },
            vcd::Value::Z  => { Bits::Z },
        }
    }
    pub fn from_vcd_vector(value: vcd::Vector) -> Self {
        let w = value.len();
        assert!(w <= 64);

        let mut v = UInt::new(0, w);
        let mut digit: u64 = 1;
        for bit in value.iter() {
            match bit {
                vcd::Value::V0 => { /* do nothing */ }
                vcd::Value::V1 => { v.value += digit; }
                vcd::Value::X  => { return Bits::X; }
                vcd::Value::Z  => { return Bits::Z; }
            }
            digit <<= 1;
        }
        Bits::V(v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bits(Bits),
    Real(f64),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueChange {
    pub time: u64,
    pub new_value: Value,
}
impl ValueChange {
    pub fn new(time: u64, new_value: Value) -> Self {
        Self{ time, new_value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueChangeStream {
    pub history: Vec<ValueChange>,
}

impl ValueChangeStream {
    pub fn new() -> Self {
        Self{ history: Vec::new() }
    }

    pub fn index_at(&self, t: u64) -> usize {
        let mut lower = 0;
        let mut upper = self.history.len();
        while 1 < upper - lower {
            assert!(lower <= upper);
            let mid = (upper + lower) / 2;
            let t_mid = self.history[mid].time;
            if t_mid < t {
                lower = mid;
            } else if t < t_mid {
                upper = mid;
            } else {
                lower = mid;
                break;
            }
        }
        lower
    }

    pub fn value_at(&self, t: u64) -> &Value {
        &self.history[self.index_at(t)].new_value
    }
    pub fn value_at_mut(&mut self, t: u64) -> &mut Value {
        let idx = self.index_at(t);
        &mut self.history[idx].new_value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub name: String,
    pub items: Vec<ScopeItem>,
}

impl Scope {
    pub fn new(name: &str) -> Self {
        Self{ name: name.to_string(), items: Vec::new() }
    }

    pub fn find_value(&self, path: &[String]) -> Option<usize> {
        if path.is_empty() {
            return None;
        }

        for item in self.items.iter() {
            match item {
                ScopeItem::Scope(s) => {
                    if s.name == path[0] {
                        return s.find_value(&path[1..path.len()]);
                    }
                }
                ScopeItem::Value(v) => {
                    if v.name == path[0] {
                        if path.len() == 1 {
                            return Some(v.index);
                        } else {
                            panic!("extraneous path: {:?}", &path[1..path.len()]);
                        }
                    }
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeValue {
    pub name: String,
    pub index: usize,
}
impl ScopeValue {
    pub fn new(name: &str, index: usize) -> Self {
        ScopeValue{ name: name.to_string(), index }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeItem {
    Scope(Scope),
    Value(ScopeValue),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeSeries {
    pub scope: Scope,
    pub values: Vec<ValueChangeStream>,
}
impl TimeSeries {
    pub fn new() -> Self {
        TimeSeries { scope: Scope::new("top"), values: Vec::new() }
    }
}
