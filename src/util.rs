use std::time::Instant;
use chrono::Local;

pub fn timer(msg: &str) -> Instant {
    let time = Instant::now();
    let date = Local::now().format("%F %r");
    println!("{} {}", date, msg);
    time
}