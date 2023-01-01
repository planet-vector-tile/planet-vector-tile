use chrono::Local;
use std::time::Instant;

pub fn timer(msg: &str) -> Instant {
    let time = Instant::now();
    let date = Local::now().format("%F %r");
    println!("<== {} ==> {}", date, msg);
    time
}

pub fn finish(t: Instant) {
    println!("Finished in {} secs.", t.elapsed().as_secs());
}
