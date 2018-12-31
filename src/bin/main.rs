use chrono::{Datelike, Timelike, Local, NaiveDate, NaiveTime, NaiveDateTime};
use env_logger::{self, Builder, Env};
use failure::Error;
#[allow(unused_imports)]
use log::{debug, info, warn, error};
//use quick_xml::Reader;
use std::{
    path::PathBuf,
    //path::Path,
};
use structopt::StructOpt;
use swinstall_stack::{
    constants::{DEFAULT_LOG_LEVEL, VERBOSE_LOG_LEVEL},
    errors::SwInstallError,
    parser::SwinstallParser,
    schemas::{ one, two, SchemaWrapper },
    utils::{ swinstall_stack_from_versionless, reader_from_file_fn },
};

#[derive(Debug, StructOpt)]
#[structopt(name = "swinst", about = "Introspect swinstall_stack, given an swinstalled file.")]
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

// Given an Option wrapped date string, convert it to a Result wrapping NaiveDate.
fn get_date(date: Option<String>) -> Result<NaiveDate, SwInstallError> {
    match date {
        Some(ref d) => {
            // construct date
            let pieces: Vec<&str> = d.split("-").collect();
            if pieces.len() != 3 {
                error!("date must be supplied using the following notation YYYY-MM-DD");
                return Err(SwInstallError::InvalidDate(d.to_string()))?;
            }
           Ok(
               NaiveDate::from_ymd(
                   pieces[0].parse::<i32>()?,
                   pieces[1].parse::<u32>()?,
                   pieces[2].parse::<u32>()?
                )
            )
        }
        None => {
           let today = Local::today();
           Ok(NaiveDate::from_ymd(today.year(), today.month(), today.day()))
        }
    }
}

fn get_time(time: Option<String>) -> Result<NaiveTime, SwInstallError> {
    match time {
        Some(ref t) => {
            let pieces: Vec<&str> = t.split(":").collect();
            if pieces.len() != 3 {
                error!("time must be supplied using the following notation: HH:MM:SS");
                return Err(SwInstallError::InvalidTime(t.to_string()))?;
            }
            Ok(
                NaiveTime::from_hms(
                    pieces[0].parse::<u32>()?,
                    pieces[1].parse::<u32>()?,
                    pieces[2].parse::<u32>()?
                )
            )
        }
        None => {
            let now = Local::now();
            Ok(NaiveTime::from_hms(now.hour(), now.minute(), now.second()))
        }
    }
}

fn main() -> Result<(), Error> {

    let opt = Opt::from_args();
    if opt.verbose {
       Builder::from_env(Env::default().default_filter_or(VERBOSE_LOG_LEVEL)).init();
    } else {
        Builder::from_env(Env::default().default_filter_or(DEFAULT_LOG_LEVEL)).init();
    }
    // create a parser
    let mut parser = SwinstallParser::new();

    // create schemas and register them with the parser
    let schema1 = SchemaWrapper::One(one::One::new());
    let schema2 = SchemaWrapper::Two(two::Two::new());

    parser.register(schema1);
    parser.register(schema2);

    // set a default schema to be used in the event that the swinstall_stack
    // does not
    parser.set_default_schema(String::from("1"));

    let date = get_date(opt.date)?;
    let time = get_time(opt.time)?;
    // now create the datetime
    let datetime_at = NaiveDateTime::new(date, time);
    let input_path = opt.input
                     .to_str()
                     .ok_or(SwInstallError::RuntimeError("unable to unwrap opt.input".to_string()))?;
    // optparse should guarantee that opt.input can be unwrapped
    let swinstall_stack = swinstall_stack_from_versionless(input_path)?;
    debug!("swinstall_stack: {}", swinstall_stack.as_str());

    let path =  parser.current_at( reader_from_file_fn(),
        swinstall_stack.as_str(), &datetime_at)?;

    println!("\npath: {}\n", path);
    Ok(())
}
