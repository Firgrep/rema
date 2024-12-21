use std::error::Error;

pub fn fatal_error(msg: &str, error: Option<Box<dyn Error>>) -> ! {
    if let Some(err) = error {
        eprintln!("{}, {}", msg, err);
    } else {
        eprintln!("{}", msg);
    }
    std::process::exit(1);
}
