mod check;
mod favicon;

use std::thread;
use std::time::Duration;

pub fn start(
    database_url: String,
    source: String,
    concurrency: usize,
    check_stations: u32,
    tcp_timeout: u64,
    max_depth: u8,
    retries: u8,
    enable_check: bool,
    pause_seconds: u64,
) {
    if enable_check {
        thread::spawn(move || loop {
            trace!("Check started.. (concurrency: {}, chunksize: {})", concurrency, check_stations);
            let result = check::dbcheck(
                database_url.clone(),
                &source,
                concurrency,
                check_stations,
                tcp_timeout,
                max_depth,
                retries,
            );
            match result {
                Ok(_)=>{},
                Err(err)=>{
                    error!("Check worker error: {}", err);
                }
            }
            thread::sleep(Duration::from_secs(pause_seconds));
        });
    }
}
