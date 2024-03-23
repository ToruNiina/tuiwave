use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufWriter;

#[allow(dead_code)]
pub fn dump(log: String) {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("ratatui.log").unwrap();

    let log = log + "\n";

    let mut writer = BufWriter::new(file);
    writer.write_all(log.as_bytes()).unwrap();
}
