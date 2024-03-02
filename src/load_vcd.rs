use crate::timeseries::*;
// use crate::log::dump;

use anyhow::Context;

use std::collections::*;

pub fn append_to_scope(scope: &mut Scope, values: &mut Vec<ValueChangeStream>, items: &Vec<vcd::ScopeItem>) -> HashMap<vcd::IdCode, usize> {
    let mut map = HashMap::new();

    for item in items.iter() {
        match item {
            vcd::ScopeItem::Var(v) => {
                let idx = values.len(); // save current idx before push

                match v.var_type {
                    vcd::VarType::Wire   => { values.push(ValueChangeStream::Bits(ValueChangeStreamImpl::new()))},
                    vcd::VarType::Reg    => { values.push(ValueChangeStream::Bits(ValueChangeStreamImpl::new()))},
                    vcd::VarType::Real   => { values.push(ValueChangeStream::Real(ValueChangeStreamImpl::new()))},
                    vcd::VarType::String => { values.push(ValueChangeStream::String(ValueChangeStreamImpl::new()))},
                    _ => {
                        // dump(format!("unsupported type {:?} found", v.var_type));
                        values.push(ValueChangeStream::Unknown)
                    }
                };
                map.insert(v.code, idx);
                scope.items.push(ScopeItem::Value(ScopeValue::new(&v.reference, idx)));
            }
            _ => {
                // do later
            }
        }
    }

    for item in items.iter() {
        match item {
            vcd::ScopeItem::Scope(s) => {
                let mut subscope = Scope::new(&s.identifier);

                let submap = append_to_scope(&mut subscope, values, &s.items);
                map.extend(submap);

                scope.items.push(ScopeItem::Scope(subscope));
            }
            vcd::ScopeItem::Var(_) => {
                // already did
            }
            _ => {
                // dump(format!("Skip: {:?}", item));
            }
        }
    }

    scope.items.sort_by(|lhs, rhs| {
        match lhs {
            ScopeItem::Scope(s1) => {
                if let ScopeItem::Scope(s2) = rhs {
                    s1.name.cmp(&s2.name)
                } else {
                    std::cmp::Ordering::Greater // value < scope
                }
            }
            ScopeItem::Value(v1) => {
                if let ScopeItem::Value(v2) = rhs {
                    v1.name.cmp(&v2.name)
                } else {
                    std::cmp::Ordering::Less
                }
            }
        }
    });
    map
}

pub fn make_value_tree(header: &vcd::Header) -> (TimeSeries, HashMap<vcd::IdCode, usize>)  {
    let mut ts = TimeSeries::new();
    let map = append_to_scope(&mut ts.scope, &mut ts.values, &header.items);
    (ts, map)
}

pub fn load_vcd<R: std::io::BufRead>(src: R) -> anyhow::Result<TimeSeries> {

    let mut parser = vcd::Parser::new(src);

    let header = parser.parse_header()?;
    let (mut ts, map) = make_value_tree(&header);

    let mut current_t = 0;

    for cmd in parser {
        let cmd = cmd?;
        match cmd {
            vcd::Command::Timestamp(t) => {
                current_t = t;
            }
            vcd::Command::ChangeScalar(i, v) => {
                let idx = map.get(&i).with_context(|| format!("ID {} NotFound", i))?;
                // idxmut does not work here
                if let ValueChangeStream::Bits(xs) = ts.values.get_mut(*idx).unwrap() {
                    if let Some(last) = xs.stream.last() {
                        if last.time == current_t {
                            xs.stream.pop();
                        }
                    }
                    xs.stream.push(ValueChange::new(current_t, Bits::from_vcd_scalar(v)));
                } else {
                    panic!("type error");
                }
            }
            vcd::Command::ChangeVector(i, v) => {
                let idx = map.get(&i).with_context(|| format!("ID {} NotFound", i))?;
                if let ValueChangeStream::Bits(xs) = ts.values.get_mut(*idx).unwrap() {
                    if let Some(last) = xs.stream.last() {
                        if last.time == current_t {
                            xs.stream.pop();
                        }
                    }
                    xs.stream.push(ValueChange::new(current_t, Bits::from_vcd_vector(v)));
                } else {
                    panic!("type error");
                }
            }
            vcd::Command::ChangeReal(i, v) => {
                let idx = map.get(&i).with_context(|| format!("ID {} NotFound", i))?;
                if let ValueChangeStream::Real(xs) = ts.values.get_mut(*idx).unwrap() {
                    if let Some(last) = xs.stream.last() {
                        if last.time == current_t {
                            xs.stream.pop();
                        }
                    }
                    xs.stream.push(ValueChange::new(current_t, v));
                } else {
                    panic!("type error");
                }
            }
            vcd::Command::ChangeString(i, v) => {
                let idx = map.get(&i).with_context(|| format!("ID {} NotFound", i))?;
                if let ValueChangeStream::String(xs) = ts.values.get_mut(*idx).unwrap() {
                    if let Some(last) = xs.stream.last() {
                        if last.time == current_t {
                            xs.stream.pop();
                        }
                    }
                    xs.stream.push(ValueChange::new(current_t, v));
                } else {
                    panic!("type error");
                }
            }
            _ => {
                // dump(format!("not supported command: {:?}", cmd));
            }
        };
    }
    Ok(ts)
}

