use crate::db::DbConnection;
use crate::db::MysqlConnection;

pub fn render(
    connection_new: &MysqlConnection,
    prefix: &str,
) -> Result<rouille::Response, Box<dyn std::error::Error>> {
    let clicks_last_hour = connection_new.get_click_count_last_hour()?;
    let stations_broken = connection_new.get_station_count_broken()?;
    let stations_working = connection_new.get_station_count_working()?;
    let stations_todo = connection_new.get_station_count_todo(24)?;
    let stations_deletable_never_worked = connection_new.get_deletable_never_working(24 * 3)?;
    let stations_deletable_were_working = connection_new.get_deletable_were_working(24 * 30)?;

    let out = format!(
        "# HELP {prefix}clicks_last_hour Clicks in the last hour
# TYPE {prefix}clicks_last_hour gauge
{prefix}clicks_last_hour {clicks_last_hour}

# HELP {prefix}stations_broken Count of stations that are broken
# TYPE {prefix}stations_broken gauge
{prefix}stations_broken {stations_broken}

# HELP {prefix}stations_working Count of stations that are working/usable
# TYPE {prefix}stations_working gauge
{prefix}stations_working {stations_working}

# HELP {prefix}stations_todo Count of stations that need are in the queue for checking
# TYPE {prefix}stations_todo gauge
{prefix}stations_todo {stations_todo}

# HELP {prefix}stations_deletable_never_worked Count of stations that are in the list for deletion and which never worked
# TYPE {prefix}stations_deletable_never_worked gauge
{prefix}stations_deletable_never_worked {stations_deletable_never_worked}

# HELP {prefix}stations_deletable_were_working Count of stations that are in the list for deletion and which worked at some point
# TYPE {prefix}stations_deletable_were_working gauge
{prefix}stations_deletable_were_working {stations_deletable_were_working}
    ",
        prefix = prefix,
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
