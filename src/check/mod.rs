mod check;
mod favicon;

use std::thread;
use std::time::Duration;
use crate::db::MysqlConnection;
use crate::db::DbConnection;

fn do_cleanup(delete: bool, database_url: &str, source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn_new_style = MysqlConnection::new(database_url)?;

    let checks_hour = conn_new_style.get_checks_todo_count(1, source)?;
    let checks_day = conn_new_style.get_checks_todo_count(24, source)?;
    let stations_broken = conn_new_style.get_station_count_broken()?;
    let stations_working = conn_new_style.get_station_count_working()?;
    let stations_todo = conn_new_style.get_station_count_todo(24)?;
    let stations_deletable_never_worked = conn_new_style.get_deletable_never_working(24 * 3)?;
    let stations_deletable_were_working = conn_new_style.get_deletable_were_working(24 * 30)?;
    if delete {
        conn_new_style.delete_never_working(24 * 3)?;
        conn_new_style.delete_were_working(24 * 30)?;
        conn_new_style.delete_old_checks(24 * 30)?;
        conn_new_style.delete_old_clicks(24 * 30)?;
    }

    info!("STATS: {} Checks/Hour, {} Checks/Day, {} Working stations, {} Broken stations, {} to do, deletable {} + {}", checks_hour, checks_day, stations_working, stations_broken, stations_todo, stations_deletable_never_worked, stations_deletable_were_working);
    Ok(())
}

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
        let result = do_cleanup(delete, &database_url2, &source2);
        if let Err(error) = result {
            error!("Error: {}", error);
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
