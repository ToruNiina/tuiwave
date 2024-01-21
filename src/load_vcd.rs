use crate::timeseries::*;

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
            vcd::ScopeItem::Var(v) => {
                // already did
            }
            _ => {
                println!("Skip: {:?}", item);
            }
        }
    }
    map
}

pub fn make_value_tree(header: &vcd::Header) -> (TimeSeries, HashMap<vcd::IdCode, usize>)  {
    let mut ts = TimeSeries::new();
    let map = append_to_scope(&mut ts.scope, &mut ts.values, &header.items);
    (ts, map)
}

pub fn load_vcd<R: std::io::BufRead>(src: R) -> std::io::Result<TimeSeries> {

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
                let idx = map.get(&i).unwrap();
                ts.values[*idx].history.push(ValueChange::new(current_t, Value::Bits(Bits::from_vcd_scalar(v))));
            }
            vcd::Command::ChangeVector(i, v) => {
                let idx = map.get(&i).unwrap();
                ts.values[*idx].history.push(ValueChange::new(current_t, Value::Bits(Bits::from_vcd_vector(v))));
            }
            vcd::Command::ChangeReal(i, v) => {
                let idx = map.get(&i).unwrap();
                ts.values[*idx].history.push(ValueChange::new(current_t, Value::Real(v)));
            }
            vcd::Command::ChangeString(i, v) => {
                let idx = map.get(&i).unwrap();
                ts.values[*idx].history.push(ValueChange::new(current_t, Value::String(v)));
            }
            _ => {
                println!("not supported command: {:?}", cmd);
            }
        }
    }
    Ok(ts)
}

