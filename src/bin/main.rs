use swinstall_stack::traits::SwinstallCurrent;
use swinstall_stack::schemas::two;
use swinstall_stack::errors::SwInstallError;
use swinstall_stack::parser::SwinstallParser;
use std::env;
use failure::Error;
use log;
use env_logger;

fn main() -> Result<(), Error> {
    env_logger::init();
    if env::args().len() < 2 {
        println!("usage: swinst file");
        return Ok(());
    }

    let arg = env::args().nth(1).ok_or(SwInstallError::RuntimeError("dont have enough elems".to_string()))?;
    println!("{}", arg);
    let mut parser = SwinstallParser::new();
    let schema2 = two::Two::new();
    parser.register(Box::new(schema2));
    parser.set_default_schema(String::from("2"));
    let path =  parser.current(arg.as_str())?;
    println!("path: {}", path);
    Ok(())
}