use crate::timeseries::*;

use anyhow::Context;

use std::collections::*;

pub fn append_to_scope(scope: &mut Scope, values: &mut Vec<ValueChangeStream>, items: &Vec<vcd::ScopeItem>) -> HashMap<vcd::IdCode, usize> {
    let mut map = HashMap::new();

    for item in items.iter() {
        match item {
            vcd::ScopeItem::Var(v) => {
                let idx = values.len();
                values.push(ValueChangeStream::new());

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
                println!("Skip: {:?}", item);
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

        if let Some((idx, v)) = match cmd {
            vcd::Command::Timestamp(t) => {
                current_t = t;
                None
            }
            vcd::Command::ChangeScalar(i, v) => {
                let idx = map.get(&i).with_context(|| format!("ID {} NotFound", i))?;
                Some((*idx, Value::Bits(Bits::from_vcd_scalar(v))))
            }
            vcd::Command::ChangeVector(i, v) => {
                let idx = map.get(&i).with_context(|| format!("ID {} NotFound", i))?;
                Some((*idx, Value::Bits(Bits::from_vcd_vector(v))))
            }
            vcd::Command::ChangeReal(i, v) => {
                let idx = map.get(&i).with_context(|| format!("ID {} NotFound", i))?;
                Some((*idx, Value::Real(v)))
            }
            vcd::Command::ChangeString(i, v) => {
                let idx = map.get(&i).with_context(|| format!("ID {} NotFound", i))?;
                Some((*idx, Value::String(v)))
            }
            _ => {
                println!("not supported command: {:?}", cmd);
                None
            }
        } {
            if let Some(last) = ts.values[idx].history.last() {
                if last.time == current_t {
                    ts.values[idx].history.pop();
                }
            }
            ts.values[idx].history.push(ValueChange::new(current_t, v));
        }
    }
    Ok(ts)
}

