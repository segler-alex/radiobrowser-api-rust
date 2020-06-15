use crate::api::api_response::ApiResponse;
use crate::db::DbConnection;
use prometheus::{Encoder, IntCounter, IntCounterVec, IntGauge, HistogramVec, TextEncoder};
use std::error::Error;
use std::convert::TryInto;

#[derive(Clone)]
pub struct RegistryLinks {
    pub timer: HistogramVec,

    pub api_calls: IntCounterVec,
    pub clicks: IntCounter,
    pub cache_hits: IntCounter,
    pub cache_misses: IntCounter,

    pub stations_broken: IntGauge,
    pub stations_working: IntGauge,
    pub stations_todo: IntGauge,
    pub stations_deletable_never_worked: IntGauge,
    pub stations_deletable_were_working: IntGauge,
    pub country_count: IntGauge,
    pub tags_count: IntGauge,
    pub language_count: IntGauge,
}

pub fn create_registry(prefix: &str) -> Result<RegistryLinks, Box<dyn Error>> {
    // Create a Counter.
    let timer = register_histogram_vec!(
        format!("{}timer", prefix),
        "Timer for the api".to_string(),
        &["method"]
    )?;
    let api_calls = register_int_counter_vec!(
        format!("{}api_calls", prefix),
        "Calls to the api".to_string(),
        &["method", "url", "status_code"]
    )?;
    let clicks = register_int_counter!(
        format!("{}station_clicks", prefix),
        "Clicks on stations".to_string()
    )?;
    let cache_hits = register_int_counter!(
        format!("{}cache_hits", prefix),
        "Calls to the api".to_string()
    )?;
    let cache_misses = register_int_counter!(format!("{}cache_misses", prefix), "def".to_string())?;

    let stations_broken = register_int_gauge!(
        format!("{}stations_broken", prefix),
        "Count of stations that are broken".to_string()
    )?;
    let stations_working = register_int_gauge!(
        format!("{}stations_working", prefix),
        "Count of stations that are working/usable".to_string()
    )?;
    let stations_todo = register_int_gauge!(
        format!("{}stations_todo", prefix),
        "Count of stations that are in the queue for checking".to_string()
    )?;
    let stations_deletable_never_worked = register_int_gauge!(
        format!("{}stations_deletable_never_worked", prefix),
        "Count of stations that are in the list for deletion and which never worked".to_string()
    )?;
    let stations_deletable_were_working = register_int_gauge!(
        format!("{}stations_deletable_were_working", prefix),
        "Count of stations that are in the list for deletion and which worked at some point"
            .to_string()
    )?;
    let country_count = register_int_gauge!(
        format!("{}country_count", prefix),
        "Count of countries".to_string()
    )?;
    let tags_count =
        register_int_gauge!(format!("{}tags_count", prefix), "Count of tags".to_string())?;
    let language_count = register_int_gauge!(
        format!("{}language_count", prefix),
        "Count of languages".to_string()
    )?;

    Ok(RegistryLinks {
        timer,
        api_calls,
        clicks,
        cache_hits,
        cache_misses,
        stations_broken,
        stations_working,
        stations_todo,
        stations_deletable_never_worked,
        stations_deletable_were_working,
        country_count,
        tags_count,
        language_count,
    })
}

pub fn render<A>(
    connection_new: &A,
    broken_stations_never_working_timeout: u64,
    broken_stations_timeout: u64,
    registry: RegistryLinks,
) -> Result<ApiResponse, Box<dyn std::error::Error>>
where
    A: DbConnection,
{
    let stations_broken = connection_new.get_station_count_broken()?;
    let stations_working = connection_new.get_station_count_working()?;
    let stations_todo = connection_new.get_station_count_todo(24)?;
    let stations_deletable_never_worked =
        connection_new.get_deletable_never_working(broken_stations_never_working_timeout)?;
    let stations_deletable_were_working =
        connection_new.get_deletable_were_working(broken_stations_timeout)?;

    let country_count = connection_new.get_country_count()?;
    let tags_count = connection_new.get_tag_count()?;
    let language_count = connection_new.get_language_count()?;

    registry.stations_broken.set(stations_broken.try_into()?);
    registry.stations_working.set(stations_working.try_into()?);
    registry.stations_todo.set(stations_todo.try_into()?);
    registry.stations_deletable_never_worked.set(stations_deletable_never_worked.try_into()?);
    registry.stations_deletable_were_working.set(stations_deletable_were_working.try_into()?);
    registry.country_count.set(country_count.try_into()?);
    registry.tags_count.set(tags_count.try_into()?);
    registry.language_count.set(language_count.try_into()?);

    // Gather the metrics.
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = prometheus::default_registry().gather();
    encoder.encode(&metric_families, &mut buffer)?;

    // Output to the standard output.
    Ok(ApiResponse::Text(String::from_utf8(buffer)?))
}
