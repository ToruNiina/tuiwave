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
