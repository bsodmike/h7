use {
    chrono::{Datelike, Local, Timelike},
    std::{collections::HashMap, fs, io, path::PathBuf, process::Command},
};

const GEN_DIR: &'static str = "gen";
const CONSTS_FILE: &'static str = "consts.rs";

fn main() {
    out_dir().unwrap();
    let mut rows = HashMap::<&'static str, (&'static str, String)>::new();
    let now = Local::now();
    rows.insert("COMPILE_TIME_YEAR", ("i32", now.year().to_string()));
    rows.insert("COMPILE_TIME_MONTH", ("u32", now.month().to_string()));
    rows.insert("COMPILE_TIME_DAY", ("u32", now.day().to_string()));
    rows.insert("COMPILE_TIME_HOUR", ("u32", now.hour().to_string()));
    rows.insert("COMPILE_TIME_MINUTE", ("u32", now.minute().to_string()));
    rows.insert("COMPILE_TIME_SECOND", ("u32", now.second().to_string()));

    let mut contents = Vec::<String>::with_capacity(rows.len());
    for (n, (t, v)) in rows {
        contents.push(format!("const {}: {} = {};", n.to_uppercase(), t, v));
    }

    fs::write(
        PathBuf::from(GEN_DIR).join(CONSTS_FILE),
        contents.join("\n"),
    )
    .unwrap();
}

fn out_dir() -> Result<(), io::Error> {
    let path = PathBuf::from(GEN_DIR);
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    fs::create_dir_all(&path)
}
