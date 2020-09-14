use crate::api::api_response::ApiResponse;
use crate::db::DbConnection;
use prometheus::{
    Encoder, HistogramVec, IntCounter, IntCounterVec, IntGauge, Registry, TextEncoder,
};
use std::convert::TryInto;
use std::error::Error;

#[derive(Clone)]
pub struct RegistryLinks {
    pub registry: Registry,
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
    let timer = register_histogram_vec!("timer", "Timer for the api", &["method"])?;
    let api_calls = IntCounterVec::new(
        opts!("api_calls", "Calls to the api"),
        &["method", "url", "status_code"],
    )?;
    let clicks = IntCounter::new("station_clicks", "Clicks on stations")?;
    let cache_hits = IntCounter::new("cache_hits", "Cache hits")?;
    let cache_misses = IntCounter::new("cache_misses", "Cache misses")?;

    let stations_broken = IntGauge::new("stations_broken", "Count of stations that are broken")?;
    let stations_working = IntGauge::new(
        "stations_working",
        "Count of stations that are working/usable",
    )?;
    let stations_todo = IntGauge::new(
        "stations_todo",
        "Count of stations that are in the queue for checking",
    )?;
    let stations_deletable_never_worked = IntGauge::new(
        "stations_deletable_never_worked",
        "Count of stations that are in the list for deletion and which never worked",
    )?;
    let stations_deletable_were_working = IntGauge::new(
        "stations_deletable_were_working",
        "Count of stations that are in the list for deletion and which worked at some point",
    )?;
    let country_count = IntGauge::new("country_count", "Count of countries")?;
    let tags_count = IntGauge::new("tags_count", "Count of tags")?;
    let language_count = IntGauge::new("language_count", "Count of languages")?;

    let registry = Registry::new_custom(Some(prefix.to_string()), None)?;
    registry.register(Box::new(timer.clone()))?;
    registry.register(Box::new(api_calls.clone()))?;
    registry.register(Box::new(clicks.clone()))?;
    registry.register(Box::new(cache_hits.clone()))?;
    registry.register(Box::new(cache_misses.clone()))?;
    registry.register(Box::new(stations_broken.clone()))?;
    registry.register(Box::new(stations_working.clone()))?;
    registry.register(Box::new(stations_todo.clone()))?;
    registry.register(Box::new(stations_deletable_never_worked.clone()))?;
    registry.register(Box::new(stations_deletable_were_working.clone()))?;
    registry.register(Box::new(country_count.clone()))?;
    registry.register(Box::new(tags_count.clone()))?;
    registry.register(Box::new(language_count.clone()))?;

    Ok(RegistryLinks {
        registry,
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
    registry
        .stations_deletable_never_worked
        .set(stations_deletable_never_worked.try_into()?);
    registry
        .stations_deletable_were_working
        .set(stations_deletable_were_working.try_into()?);
    registry.country_count.set(country_count.try_into()?);
    registry.tags_count.set(tags_count.try_into()?);
    registry.language_count.set(language_count.try_into()?);

    // Gather the metrics.
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = registry.registry.gather();
    encoder.encode(&metric_families, &mut buffer)?;

    // Output to the standard output.
    Ok(ApiResponse::Text(String::from_utf8(buffer)?))
}
