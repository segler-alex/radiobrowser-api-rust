pub mod models;

mod check;
mod db;
mod favicon;

use std::thread;
use std::time::Duration;

pub fn start(
    database_url: String,
    source: String,
    delete: bool,
    concurrency: usize,
    check_stations: u32,
    useragent: String,
    tcp_timeout: u32,
    max_depth: u8,
    retries: u8,
    favicon: bool,
    enable_check: bool,
    pause_seconds: u64,
) {
    let database_url2 = database_url.clone();
    let source2 = source.clone();
    thread::spawn(move || loop {
        let conn = db::new(&database_url2);
        match conn {
            Ok(conn) => {
                let checks_hour = db::get_checks(&conn, 1, &source2);
                let checks_day = db::get_checks(&conn, 24, &source2);
                let stations_broken = db::get_station_count_broken(&conn);
                let stations_working = db::get_station_count_working(&conn);
                let stations_todo = db::get_station_count_todo(&conn, 24);
                let stations_deletable_never_worked =
                    db::get_deletable_never_working(&conn, 24 * 3);
                let stations_deletable_were_working =
                    db::get_deletable_were_working(&conn, 24 * 30);
                if delete {
                    db::delete_never_working(&conn, 24 * 3);
                    db::delete_were_working(&conn, 24 * 30);
                    db::delete_old_checks(&conn, 24 * 30);
                    db::delete_old_clicks(&conn, 24 * 30);
                }

                info!("STATS: {} Checks/Hour, {} Checks/Day, {} Working stations, {} Broken stations, {} to do, deletable {} + {}", checks_hour, checks_day, stations_working, stations_broken, stations_todo, stations_deletable_never_worked, stations_deletable_were_working);
            }
            Err(e) => {
                error!("Database connection error {}", e);
            }
        }
        thread::sleep(Duration::from_secs(3600));
    });

    if enable_check {
        thread::spawn(move || loop {
            let _checked_count = check::dbcheck(
                &database_url,
                &source,
                concurrency,
                check_stations,
                &useragent,
                tcp_timeout,
                max_depth,
                retries,
                favicon,
            );
            thread::sleep(Duration::from_secs(pause_seconds));
        });
    }
}
