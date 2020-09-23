use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::cmp::{Ordering, PartialEq};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, Serialize)]
struct Record {
    #[serde(with = "chrono::serde::ts_seconds")]
    timestamp: chrono::DateTime<chrono::Utc>,
    systolic: u32,
    diastolic: u32,
    pulse: u32,
}

impl Display for Record {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let local_timestamp: chrono::DateTime<chrono::Local> =
            chrono::DateTime::from(self.timestamp);
        write!(
            f,
            "{}\tBP: {}/{}\tPulse: {}",
            local_timestamp.format("%Y-%m-%d %I:%M%P"),
            self.systolic,
            self.diastolic,
            self.pulse
        )
    }
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Record) -> Option<Ordering> {
        Some(self.timestamp.cmp(&other.timestamp))
    }
}

#[derive(Debug, StructOpt)]
struct RecordOpts {
    #[structopt(long, help = "Systolic pressure")]
    top: u32,
    #[structopt(long, help = "Diastolic pressure")]
    bottom: u32,
    #[structopt(long, help = "Pulse in bpm")]
    pulse: u32,
}

#[derive(Debug, StructOpt)]
struct ReportOpts {
    #[structopt(default_value = "10", long)]
    limit: usize,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "bloodpressure", about = "Record and report my blood pressure")]
enum Command {
    Record(RecordOpts),
    Report(ReportOpts),
    ShowPath,
}

fn get_data_paths() -> Result<(PathBuf, PathBuf)> {
    if let Some(p) = dirs::data_local_dir() {
        let dd = p.join("bloodpressure");
        let df = dd.join("data.csv");
        Ok((dd, df))
    } else {
        bail!("Could not compute path!");
    }
}

fn do_record(opts: RecordOpts) -> Result<()> {
    let (data_dir, data_path) = get_data_paths()?;
    fs::create_dir_all(data_dir)?;
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(data_path)?;

    let record = Record {
        timestamp: chrono::Utc::now(),
        systolic: opts.top,
        diastolic: opts.bottom,
        pulse: opts.pulse,
    };

    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);
    writer.serialize(record)?;
    writer.flush()?;
    Ok(())
}

fn do_report(opts: ReportOpts) -> Result<()> {
    let (data_dir, data_path) = get_data_paths()?;
    fs::create_dir_all(data_dir)?;
    let file = fs::OpenOptions::new().read(true).open(data_path)?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);
    let mut records: Vec<Record> = vec![];

    for result in reader.deserialize() {
        records.push(result?);
    }
    records.sort();
    records.reverse();

    for record in records.iter().take(opts.limit) {
        println!("{}", record);
    }

    Ok(())
}

fn do_show_path() -> Result<()> {
    let (_, data_path) = get_data_paths()?;
    println!("Data Path: {:?}", data_path);
    Ok(())
}

fn main() -> Result<()> {
    let command = Command::from_args();
    match command {
        Command::Record(opts) => do_record(opts),
        Command::Report(opts) => do_report(opts),
        Command::ShowPath => do_show_path(),
    }
}
