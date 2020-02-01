use crate::db::connect;
use std::thread;
use std::time::Duration;

fn do_cleanup(
    delete: bool,
    database_url: String,
    source: &str,
    click_valid_timeout: u64,
    broken_stations_never_working_timeout: u64,
    broken_stations_timeout: u64,
    checks_timeout: u64,
    clicks_timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn_new_style = connect(database_url)?;

    let checks_hour = conn_new_style.get_checks_todo_count(1, source)?;
    let checks_day = conn_new_style.get_checks_todo_count(24, source)?;
    let stations_broken = conn_new_style.get_station_count_broken()?;
    let stations_working = conn_new_style.get_station_count_working()?;
    let stations_todo = conn_new_style.get_station_count_todo(24)?;
    let stations_deletable_never_worked =
        conn_new_style.get_deletable_never_working(broken_stations_never_working_timeout)?;
    let stations_deletable_were_working =
        conn_new_style.get_deletable_were_working(broken_stations_timeout)?;
    if delete {
        conn_new_style.delete_never_working(broken_stations_never_working_timeout)?;
        conn_new_style.delete_were_working(broken_stations_timeout)?;
        conn_new_style.delete_old_checks(checks_timeout)?;
        conn_new_style.delete_old_clicks(clicks_timeout)?;
    }

    conn_new_style.update_stations_clickcount()?;
    conn_new_style.remove_unused_ip_infos_from_stationclicks(click_valid_timeout)?;
    conn_new_style.remove_illegal_icon_links()?;

    info!("STATS: {} Checks/Hour, {} Checks/Day, {} Working stations, {} Broken stations, {} to do, deletable {} + {}", checks_hour, checks_day, stations_working, stations_broken, stations_todo, stations_deletable_never_worked, stations_deletable_were_working);
    Ok(())
}

pub fn start(
    database_url: String,
    source: String,
    delete: bool,
    pause_seconds: u64,
    clicks_valid_timeout: u64,
    broken_stations_never_working_timeout: u64,
    broken_stations_timeout: u64,
    checks_timeout: u64,
    clicks_timeout: u64,
) {
    thread::spawn(move || loop {
        let result = do_cleanup(
            delete,
            database_url.clone(),
            &source,
            clicks_valid_timeout,
            broken_stations_never_working_timeout,
            broken_stations_timeout,
            checks_timeout,
            clicks_timeout,
        );
        if let Err(error) = result {
            error!("Error: {}", error);
        }
        thread::sleep(Duration::from_secs(pause_seconds));
    });
}
