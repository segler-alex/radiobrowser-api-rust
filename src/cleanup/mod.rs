use crate::DbConnection;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Clone, Deserialize)]
struct DataMappingItem {
    from: String,
    to: String,
}

pub fn do_cleanup<C>(
    delete: bool,
    mut conn_new_style: C,
    click_valid_timeout: u64,
    broken_stations_never_working_timeout: u64,
    broken_stations_timeout: u64,
    checks_timeout: u64,
    clicks_timeout: u64,
) -> Result<(), Box<dyn Error>> where C: DbConnection {
    let checks_hour = conn_new_style.get_station_count_todo(1)?;
    let checks_day = conn_new_style.get_station_count_todo(24)?;
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
        conn_new_style.delete_removed_from_history()?;
        conn_new_style.delete_unused_streaming_servers(24 * 60 * 60)?;
    }

    conn_new_style.update_stations_clickcount()?;
    conn_new_style.remove_unused_ip_infos_from_stationclicks(click_valid_timeout)?;
    conn_new_style.calc_country_field()?;

    info!("STATS: {} Checks/Hour, {} Checks/Day, {} Working stations, {} Broken stations, {} to do, deletable {} + {}", checks_hour, checks_day, stations_working, stations_broken, stations_todo, stations_deletable_never_worked, stations_deletable_were_working);
    Ok(())
}
