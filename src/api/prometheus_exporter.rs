use crate::db::DbConnection;

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

pub fn render<A>(
    connection_new: &A,
    prefix: &str,
    broken_stations_never_working_timeout: u64,
    broken_stations_timeout: u64,
    counter_all: Arc<AtomicUsize>,
    counter_clicks: Arc<AtomicUsize>,
) -> Result<rouille::Response, Box<dyn std::error::Error>> where A: DbConnection {
    let stations_broken = connection_new.get_station_count_broken()?;
    let stations_working = connection_new.get_station_count_working()?;
    let stations_todo = connection_new.get_station_count_todo(24)?;
    let stations_deletable_never_worked = connection_new.get_deletable_never_working(broken_stations_never_working_timeout)?;
    let stations_deletable_were_working = connection_new.get_deletable_were_working(broken_stations_timeout)?;

    let country_count = connection_new.get_country_count()?;
    let tags_count = connection_new.get_tag_count()?;
    let language_count = connection_new.get_language_count()?;

    let station_clicks = counter_clicks.load(Ordering::Relaxed);
    let api_calls = counter_all.load(Ordering::Relaxed);

    let out = format!(
        "# HELP {prefix}station_clicks Clicks on stations
# TYPE {prefix}station_clicks counter
{prefix}station_clicks {station_clicks}

# HELP {prefix}api_calls Calls to the api
# TYPE {prefix}api_calls counter
{prefix}api_calls {api_calls}

# HELP {prefix}stations_broken Count of stations that are broken
# TYPE {prefix}stations_broken gauge
{prefix}stations_broken {stations_broken}

# HELP {prefix}stations_working Count of stations that are working/usable
# TYPE {prefix}stations_working gauge
{prefix}stations_working {stations_working}

# HELP {prefix}stations_todo Count of stations that are in the queue for checking
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
        stations_broken = stations_broken,
        stations_working = stations_working,
        stations_todo = stations_todo,
        stations_deletable_never_worked = stations_deletable_never_worked,
        stations_deletable_were_working = stations_deletable_were_working,
        country_count = country_count,
        tags_count = tags_count,
        language_count = language_count,
        station_clicks = station_clicks,
        api_calls = api_calls,
    );

    // Output to the standard output.
    Ok(rouille::Response::text(out).with_status_code(200))
}
