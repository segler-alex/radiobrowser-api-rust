use crate::db::DbConnection;
use crate::db::MysqlConnection;

pub fn render(
    connection_new: &MysqlConnection,
) -> Result<rouille::Response, Box<dyn std::error::Error>> {
    let clicks_last_hour = connection_new.get_click_count_last_hour()?;
    let stations_broken = connection_new.get_station_count_broken()?;
    let stations_working = connection_new.get_station_count_working()?;
    let stations_todo = connection_new.get_station_count_todo(24)?;
    let stations_deletable_never_worked = connection_new.get_deletable_never_working(24 * 3)?;
    let stations_deletable_were_working = connection_new.get_deletable_were_working(24 * 30)?;

    let out = format!(
        "# HELP clicks_last_hour Clicks in the last hour
# TYPE clicks_last_hour gauge
clicks_last_hour {clicks_last_hour}

# HELP stations_broken Count of stations that are broken
# TYPE stations_broken gauge
stations_broken {stations_broken}

# HELP stations_working Count of stations that are working/usable
# TYPE stations_working gauge
stations_working {stations_working}

# HELP stations_todo Count of stations that need are in the queue for checking
# TYPE stations_todo gauge
stations_todo {stations_todo}

# HELP stations_deletable_never_worked Count of stations that are in the list for deletion and which never worked
# TYPE stations_deletable_never_worked gauge
stations_deletable_never_worked {stations_deletable_never_worked}

# HELP stations_deletable_were_working Count of stations that are in the list for deletion and which worked at some point
# TYPE stations_deletable_were_working gauge
stations_deletable_were_working {stations_deletable_were_working}
    ",
        clicks_last_hour = clicks_last_hour,
        stations_broken = stations_broken,
        stations_working = stations_working,
        stations_todo = stations_todo,
        stations_deletable_never_worked = stations_deletable_never_worked,
        stations_deletable_were_working = stations_deletable_were_working,
    );

    // Output to the standard output.
    Ok(rouille::Response::text(out).with_status_code(200))
}
