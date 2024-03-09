#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UInt {
    pub value: u128,
    pub width: usize,
}

impl UInt {
    pub fn new(value: u128, width: usize) -> UInt {
        UInt{value, width}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bits {
    B(bool),
    V(UInt),
    X,
    Z,
}

impl Bits {
    pub fn from_vcd_scalar(value: vcd::Value) -> Self {
        match value {
            vcd::Value::V0 => { Bits::B(false) },
            vcd::Value::V1 => { Bits::B(true)  },
            vcd::Value::X  => { Bits::X },
            vcd::Value::Z  => { Bits::Z },
        }
    }
    pub fn from_vcd_vector(value: vcd::Vector) -> Self {
        let w = value.len();
        assert!(w <= 128);

        if w == 0 {
            return Bits::B(false);
        }
        if w == 1 {
            return match value.get(0).unwrap() {
                vcd::Value::V0 => { Bits::B(false) },
                vcd::Value::V1 => { Bits::B(true)  },
                vcd::Value::X  => { Bits::X },
                vcd::Value::Z  => { Bits::Z },
            };
        }

        let bits: Vec<vcd::Value> = value.iter().collect();
        let mut v = UInt::new(0, w);
        let mut digit: u128 = 1;
        for bit in bits.iter().rev() {
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
pub struct ValueChange<T: std::fmt::Debug + Clone + PartialEq> {
    pub time: u64,
    pub new_value: T,
}
impl<T: std::fmt::Debug + Clone + PartialEq> ValueChange<T> {
    pub fn new(time: u64, new_value: T) -> Self {
        Self{ time, new_value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueChangeStreamImpl<T: std::fmt::Debug + Clone + PartialEq> {
    pub stream: Vec<ValueChange<T>>
}

impl<T: std::fmt::Debug + Clone + PartialEq> ValueChangeStreamImpl<T> {

    pub fn new() -> Self {
        Self{ stream: Vec::new() }
    }

    pub fn change_before(&self, t: u64) -> Option<usize> {
        if let Some(first) = self.stream.first() {
            if t < first.time {
                return None;
            }
        } else {
            return None; // empty!
        }

        if t == 0 {
            return Some(0); // first.time <= t, and t is u64
        }

        let mut lower = 0;
        let mut upper = self.stream.len();
        while 1 < upper - lower {
            assert!(lower <= upper);
            let mid = (upper + lower) / 2;
            let t_mid = self.stream[mid].time;
            if t_mid < t {
                lower = mid;
            } else if t < t_mid {
                upper = mid;
            } else {
                lower = mid;
                break;
            }
        }
        Some(lower)
    }

    pub fn change_after(&self, t: u64) -> Option<usize> {
        if let Some(last) = self.stream.last() {
            if last.time < t {
                return None;
            }
        } else {
            return None; // empty!
        }

        let mut lower = 0;
        let mut upper = self.stream.len();
        while 1 < upper - lower {
            assert!(lower <= upper);
            let mid = (upper + lower) / 2;
            let t_mid = self.stream[mid].time;
            if t_mid < t {
                lower = mid;
            } else if t < t_mid {
                upper = mid;
            } else { // t == t_mid
                upper = mid+1;
                break
            }
        }
        Some(upper)
    }

    pub fn last_change_time(&self) -> u64 {
        self.stream.iter().map(|x| x.time).max().unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueChangeStream {
    Bits  (ValueChangeStreamImpl<Bits>),
    Real  (ValueChangeStreamImpl<f64>),
    String(ValueChangeStreamImpl<String>),
    Unknown,
}

impl ValueChangeStream {
    pub fn last_change_time(&self) -> u64 {
        match self {
            Self::Bits(xs)   => { xs.last_change_time() }
            Self::Real(xs)   => { xs.last_change_time() }
            Self::String(xs) => { xs.last_change_time() }
            _ => { 0 }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub name: String,
    pub items: Vec<ScopeItem>,
    pub open: bool, // open in sidebar tree (UI)
}

impl Scope {
    pub fn new(name: &str) -> Self {
        Self{ name: name.to_string(), items: Vec::new(), open: true }
    }

    pub fn should_be_rendered(&self) -> bool {
        self.items.iter().map(|x| x.should_be_rendered()).reduce(|acc, e| acc || e).unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeValue {
    pub name: String,
    pub index: usize,
    pub render: bool, // (UI)
}
impl ScopeValue {
    pub fn new(name: &str, index: usize) -> Self {
        ScopeValue{ name: name.to_string(), index, render: true }
    }

    pub fn should_be_rendered(&self) -> bool {
        self.render
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeItem {
    Scope(Scope),
    Value(ScopeValue),
}

impl ScopeItem {
    pub fn should_be_rendered(&self) -> bool {
        match self {
            ScopeItem::Scope(s) => {s.should_be_rendered()},
            ScopeItem::Value(v) => {v.should_be_rendered()},
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeSeries {
    pub scope: Scope,
    pub values: Vec<ValueChangeStream>,
    pub time_scale: (u32, String), // (100, "us")
}

impl TimeSeries {
    pub fn new() -> Self {
        TimeSeries { scope: Scope::new("top"), values: Vec::new(), time_scale: (1, "tau".to_string()) }
    }
}
