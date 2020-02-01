use crate::db::DbConnection;

pub fn render<A>(
    connection_new: &A,
    prefix: &str,
) -> Result<rouille::Response, Box<dyn std::error::Error>> where A: DbConnection {
    let clicks_last_hour = connection_new.get_click_count_last_hour()?;
    let clicks_last_day = connection_new.get_click_count_last_day()?;
    let stations_broken = connection_new.get_station_count_broken()?;
    let stations_working = connection_new.get_station_count_working()?;
    let stations_todo = connection_new.get_station_count_todo(24)?;
    let stations_deletable_never_worked = connection_new.get_deletable_never_working(24 * 3)?;
    let stations_deletable_were_working = connection_new.get_deletable_were_working(24 * 30)?;

    let country_count = connection_new.get_country_count()?;
    let tags_count = connection_new.get_tag_count()?;
    let language_count = connection_new.get_language_count()?;

    let out = format!(
        "# HELP {prefix}clicks_last_hour Clicks in the last hour
# TYPE {prefix}clicks_last_hour gauge
{prefix}clicks_last_hour {clicks_last_hour}

# HELP {prefix}clicks_last_day Clicks in the last day
# TYPE {prefix}clicks_last_day gauge
{prefix}clicks_last_day {clicks_last_day}

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

# HELP {prefix}country_count Count of countries
# TYPE {prefix}country_count gauge
{prefix}country_count {country_count}

# HELP {prefix}tags_count Count of tags
# TYPE {prefix}tags_count gauge
{prefix}tags_count {tags_count}

# HELP {prefix}language_count Count of languages
# TYPE {prefix}language_count gauge
{prefix}language_count {language_count}
    ",
        prefix = prefix,
        clicks_last_hour = clicks_last_hour,
        clicks_last_day = clicks_last_day,
        stations_broken = stations_broken,
        stations_working = stations_working,
        stations_todo = stations_todo,
        stations_deletable_never_worked = stations_deletable_never_worked,
        stations_deletable_were_working = stations_deletable_were_working,
        country_count = country_count,
        tags_count = tags_count,
        language_count = language_count,
    );

    // Output to the standard output.
    Ok(rouille::Response::text(out).with_status_code(200))
}
