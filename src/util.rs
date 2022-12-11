use chrono::Local;
use std::time::Instant;

pub fn timer(msg: &str) -> Instant {
    let time = Instant::now();
    let date = Local::now().format("%F %r");
    println!("<== {} ==> {}", date, msg);
    time
}
