use crate::db::connect;

pub fn do_cleanup(
    delete: bool,
    database_url: String,
    click_valid_timeout: u64,
    broken_stations_never_working_timeout: u64,
    broken_stations_timeout: u64,
    checks_timeout: u64,
    clicks_timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn_new_style = connect(database_url)?;

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
    }

    conn_new_style.clean_urls("Station", "StationUuid", "Url", false)?;
    conn_new_style.clean_urls("Station", "StationUuid", "Homepage", true)?;
    conn_new_style.clean_urls("Station", "StationUuid", "UrlCache", true)?;
    conn_new_style.clean_urls("StationHistory", "StationUuid", "Url", false)?;
    conn_new_style.clean_urls("StationHistory", "StationUuid", "Homepage", true)?;
    conn_new_style.update_stations_clickcount()?;
    conn_new_style.remove_unused_ip_infos_from_stationclicks(click_valid_timeout)?;
    conn_new_style.remove_illegal_icon_links()?;
    conn_new_style.calc_country_field()?;

    info!("STATS: {} Checks/Hour, {} Checks/Day, {} Working stations, {} Broken stations, {} to do, deletable {} + {}", checks_hour, checks_day, stations_working, stations_broken, stations_todo, stations_deletable_never_worked, stations_deletable_were_working);
    Ok(())
}
