use chrono::{Datelike, Timelike, Local, NaiveDate, NaiveTime, NaiveDateTime};
use env_logger;
use failure::Error;
use log::{debug, info, warn, error};
use std::{
    env,
    path::PathBuf
};
use structopt::StructOpt;
use swinstall_stack::{
    errors::SwInstallError,
    parser::SwinstallParser,
    schemas::two,
    traits::SwinstallCurrent,
};


#[derive(Debug, StructOpt)]
#[structopt(name = "swinst", about = "Introspect swinstall_stack")]
struct Opt {
    /// Activate debug mode
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
    /// Supply explicit date, in the form YYYY-MM-DD
    #[structopt(short = "d", long = "date")]
    date: Option<String>,
    /// Supply explicit time, in the form HH:MM:SS
    #[structopt(short = "t", long = "time")]
    time: Option<String>,
    #[structopt(parse(from_os_str))]
    input:  PathBuf
}


fn main() -> Result<(), Error> {
    env_logger::init();

    let opt = Opt::from_args();

    let mut parser = SwinstallParser::new();

    // create schemas
    let schema2 = two::Two::new();
    parser.register(Box::new(schema2));

    parser.set_default_schema(String::from("2"));

    let date = match opt.date {
        Some(ref d) => {
            // construct date
            let pieces: Vec<&str> = d.split("-").collect();
            if pieces.len() != 3 {
                error!("date must be supplied using the following notation YYYY-MM-DD");
                return Err(SwInstallError::InvalidDate(d.to_string()))?;
            }
            NaiveDate::from_ymd(pieces[0].parse::<i32>()?, pieces[1].parse::<u32>()?, pieces[2].parse::<u32>()?)
        }
        None => {
           let today = Local::today();
           NaiveDate::from_ymd(today.year(), today.month(), today.day())
        }
    };

    let time = match opt.time {
        Some(ref t) => {
            let pieces: Vec<&str> = t.split(":").collect();
            if pieces.len() != 3 {
                error!("time must be supplied using the following notation: HH:MM:SS");
                return Err(SwInstallError::InvalidTime(t.to_string()))?;
            }
            NaiveTime::from_hms(pieces[0].parse::<u32>()?, pieces[1].parse::<u32>()?, pieces[2].parse::<u32>()?)
        }
        None => {
            let now = Local::now();
            NaiveTime::from_hms(now.hour(), now.minute(), now.second())
        }
    };
    // now create the datetime
    let datetime_at = NaiveDateTime::new(date, time);

    // optparse should guarantee that opt.input can be unwrapped
    let path =  parser.current_at(opt.input.to_str().unwrap(), &datetime_at)?;
    println!("path: {}", path);
    Ok(())
}
