use crate::check::favicon::get_best_icon;
use crate::config::get_cache_tags_replace;
use crate::config::get_cache_language_replace;
use crate::config::get_cache_language_to_code;
use crate::db::models::DbStationItem;
use crate::db::models::DbStreamingServerNew;
use crate::db::models::StationCheckItemNew;
use crate::db::models::StationCheckStepItemNew;
use crate::db::DbConnection;
use av_stream_info_rust;
use av_stream_info_rust::StreamCheckResult;
use av_stream_info_rust::UrlType;
use rayon::prelude::*;
use reqwest::blocking::Client;
use std;
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;
use url::Url;
use uuid::Uuid;
use website_icon_extract::ImageLink;

#[derive(Clone, Debug)]
pub struct StationOldNew {
    pub station: DbStationItem,
    pub check: StationCheckItemNew,
    pub steps: Vec<StationCheckStepItemNew>,
}

/// returns list of
/// (stepuuid, parentstepuuid, checkitemnew)
fn flatten_check_result(
    stationuuid: String,
    checkuuid: String,
    result: StreamCheckResult,
    parent: Option<String>,
    source: &str,
    timing_ms: u128,
) -> (Vec<StationCheckStepItemNew>, Option<StationCheckItemNew>) {
    let mut list = vec![];
    let url = result.url().to_string();
    let mut found_working: Option<StationCheckItemNew> = None;
    match result.info {
        Ok(info) => {
            match info {
                UrlType::Stream(info) => {
                    found_working = Some(StationCheckItemNew::working(
                        stationuuid.clone(),
                        checkuuid.to_string(),
                        source.to_string(),
                        timing_ms,
                        url.to_string(),
                        info,
                    ));
                    let new_item = StationCheckStepItemNew {
                        stepuuid: Uuid::new_v4().to_hyphenated().to_string(),
                        parent_stepuuid: parent,
                        checkuuid,
                        stationuuid,
                        url,
                        urltype: Some("STREAM".to_string()),
                        error: None,
                    };
                    list.push(new_item);
                }
                UrlType::Redirect(item) => {
                    let stepuuid = Uuid::new_v4().to_hyphenated().to_string();
                    let new_item = StationCheckStepItemNew {
                        stepuuid: stepuuid.clone(),
                        parent_stepuuid: parent,
                        checkuuid: checkuuid.clone(),
                        stationuuid: stationuuid.clone(),
                        url,
                        urltype: Some("REDIRECT".to_string()),
                        error: None,
                    };
                    list.push(new_item);
                    let (ret_list, ret_found) = flatten_check_result(
                        stationuuid,
                        checkuuid,
                        *item,
                        Some(stepuuid),
                        source,
                        timing_ms,
                    );
                    list.extend(ret_list);
                    if ret_found.is_some() {
                        found_working = ret_found;
                    }
                }
                UrlType::PlayList(playlist) => {
                    let stepuuid = Uuid::new_v4().to_hyphenated().to_string();
                    let new_item = StationCheckStepItemNew {
                        stepuuid: stepuuid.clone(),
                        parent_stepuuid: parent,
                        checkuuid: checkuuid.clone(),
                        stationuuid: checkuuid.clone(),
                        url,
                        urltype: Some("PLAYLIST".to_string()),
                        error: None,
                    };
                    list.push(new_item);
                    for playlist_item in playlist {
                        let (ret_list, ret_found) = flatten_check_result(
                            stationuuid.clone(),
                            checkuuid.clone(),
                            playlist_item,
                            Some(stepuuid.clone()),
                            source,
                            timing_ms,
                        );
                        list.extend(ret_list);
                        if ret_found.is_some() {
                            found_working = ret_found;
                        }
                    }
                }
            };
        }
        Err(err) => {
            let new_item = StationCheckStepItemNew {
                stepuuid: Uuid::new_v4().to_hyphenated().to_string(),
                parent_stepuuid: parent,
                checkuuid,
                stationuuid,
                url,
                urltype: None,
                error: Some(err.to_string()),
            };
            list.push(new_item);
        }
    }
    (list, found_working)
}

fn dbcheck_internal(
    station: DbStationItem,
    source: &str,
    timeout: u64,
    max_depth: u8,
    retries: u8,
) -> StationOldNew {
    let checkuuid = Uuid::new_v4().to_hyphenated().to_string();
    let now = Instant::now();
    trace!("Check started: {} - {}", station.stationuuid, station.name);
    let checks: StreamCheckResult =
        av_stream_info_rust::check_tree(&station.url, timeout as u32, max_depth, retries, true);
    let timing_ms = now.elapsed().as_millis();
    let (steps, check) = flatten_check_result(
        station.stationuuid.clone(),
        checkuuid.clone(),
        checks,
        None,
        source,
        timing_ms,
    );

    match check {
        Some(check) => StationOldNew {
            station,
            check,
            steps,
        },
        None => {
            let check = StationCheckItemNew::broken(
                station.stationuuid.clone(),
                checkuuid,
                source.to_string(),
                timing_ms,
            );
            StationOldNew {
                station,
                check,
                steps,
            }
        }
    }
}

pub fn dbcheck<C>(
    mut conn: C,
    source: &str,
    concurrency: usize,
    stations_count: u32,
    timeout: u64,
    max_depth: u8,
    retries: u8,
    add_streaming_servers: bool,
    recheck_existing_favicon: bool,
    enable_extract_favicon: bool,
    favicon_size_min: usize,
    favicon_size_max: usize,
    favicon_size_optimum: usize,
) -> Result<usize, Box<dyn std::error::Error>>
where
    C: DbConnection,
{
    let stations = conn.get_stations_to_check(24, stations_count)?;
    let checked_count = stations.len();
    let agent = "radiobrowser-api-rust/0.1.0";

    let client = Client::builder()
        .user_agent(agent)
        .timeout(Duration::from_secs(timeout))
        .build()?;

    let languages_cache: HashMap<String, String> = match get_cache_language_replace() {
        Some(mutex) => match mutex.lock() {
            Ok(map) => map.clone(),
            Err(err) => {
                warn!(
                    "Unable to get language mapping cache from shared memory: {}",
                    err
                );
                HashMap::new()
            }
        },
        None => HashMap::new(),
    };

    let tags_cache: HashMap<String, String> = match get_cache_tags_replace() {
        Some(mutex) => match mutex.lock() {
            Ok(map) => map.clone(),
            Err(err) => {
                warn!(
                    "Unable to get tag mapping cache from shared memory: {}",
                    err
                );
                HashMap::new()
            }
        },
        None => HashMap::new(),
    };

    let language_to_code_cache: HashMap<String, String> = match get_cache_language_to_code() {
        Some(mutex) => match mutex.lock() {
            Ok(map) => map.clone(),
            Err(err) => {
                warn!(
                    "Unable to get language mapping cache from shared memory: {}",
                    err
                );
                HashMap::new()
            }
        },
        None => HashMap::new(),
    };

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build()?;
    let results: Vec<_> = pool.install(|| {
        stations
            .into_par_iter()
            .map(|mut station| {
                // check current favicon
                if !station.favicon.is_empty() && recheck_existing_favicon {
                    trace!(
                        "checking favicon {} '{}'",
                        station.stationuuid,
                        station.favicon
                    );
                    let request = client.head(&station.favicon).send();
                    //let link = ImageLink::new(&station.favicon, agent, timeout);
                    if request.is_err() {
                        trace!(
                            "removed favicon {} '{}'",
                            station.stationuuid, station.favicon
                        );
                        // reset favicon, it could not be loaded
                        station.set_favicon(String::new());
                    }
                }
                station
            })
            .map(|mut station| {
                if station.favicon.is_empty() && enable_extract_favicon {
                    trace!("searching favicon {}", station.stationuuid);
                    let links = ImageLink::from_website(&station.homepage, agent, timeout);
                    if let Ok(links) = links {
                        let icon = get_best_icon(
                            links,
                            favicon_size_optimum,
                            favicon_size_min,
                            favicon_size_max,
                        );
                        if let Some(icon) = icon {
                            station.set_favicon(icon.url.to_string());
                            trace!(
                                "added favicon {} '{}'",
                                station.stationuuid,
                                station.favicon
                            );
                        }
                    }
                }
                station
            })
            .map(|mut station| {
                let lang_copy = station.language.clone();
                let mut lang_trimmed: Vec<&str> = lang_copy
                    .split(",")
                    .by_ref()
                    .map(|item| item.trim())
                    .filter(|item| !item.is_empty())
                    .map(|item| match languages_cache.get(&item.to_string()) {
                        Some(item_replaced) => {
                            trace!("replace '{}' -> '{}'", item, item_replaced);
                            item_replaced
                        }
                        None => item,
                    })
                    .map(|item| item.trim())
                    .filter(|item| !item.is_empty())
                    .collect();
                lang_trimmed.sort();
                lang_trimmed.dedup();
                station.set_language(lang_trimmed.join(","));
                station
            })
            .map(|mut station| {
                let tags_copy = station.tags.clone();
                let mut tags_trimmed: Vec<&str> = tags_copy
                    .split(",")
                    .by_ref()
                    .map(|item| item.trim())
                    .filter(|item| !item.is_empty())
                    .map(|item| match tags_cache.get(&item.to_string()) {
                        Some(item_replaced) => {
                            trace!("replace '{}' -> '{}'", item, item_replaced);
                            item_replaced
                        }
                        None => item,
                    })
                    .map(|item| item.trim())
                    .filter(|item| !item.is_empty())
                    .collect();
                tags_trimmed.sort();
                tags_trimmed.dedup();
                station.set_tags(tags_trimmed.join(","));
                station
            })
            .map(|mut station| {
                let language = station.language.clone();
                // convert each language to code
                let mut lang_trimmed: Vec<&str> = language
                    .split(",")
                    .by_ref()
                    .map(|item| item.trim())
                    .filter(|item| !item.is_empty())
                    .filter_map(|item| {
                        let detected = language_to_code_cache.get(item);
                        detected
                    })
                    .map(|item| item.as_ref())
                    .collect();
                lang_trimmed.sort();
                lang_trimmed.dedup();
                let codes = station.languagecodes.clone();
                // cleanup current codes
                let mut codes_trimmed: Vec<&str> = codes
                    .split(",")
                    .by_ref()
                    .map(|item| item.trim())
                    .filter(|item| !item.is_empty())
                    .collect();
                for new_lang in lang_trimmed.drain(..) {
                    if !codes_trimmed.contains(&new_lang) {
                        codes_trimmed.push(new_lang);
                    }
                }
                station.set_languagecodes(codes_trimmed.join(","));
                station
            })
            .map(|mut station| {
                station.set_homepage(Url::parse(&station.homepage).map(|u|u.to_string()).unwrap_or_default());
                station.set_url(Url::parse(&station.url).map(|u|u.to_string()).unwrap_or_default());
                station
            })
            .map(|station| dbcheck_internal(station, source, timeout, max_depth, retries))
            //.map(|mut diff| {
            //    diff.station.set_bitrate(diff.check.bitrate);
            //    diff.station.set_codec(&diff.check.codec);
            //    diff.station.set_hls(diff.check.hls);
            //    diff.station.set_last_check_ok(diff.check.check_ok);
            //    diff
            //})
            .collect()
    });

    // do real insert
    let mut checks = vec![];
    let mut steps = vec![];
    for result in results {
        checks.push(result.check);
        steps.extend(result.steps);

        if result.station.get_changed() {
            debug!("changed {}", result.station.stationuuid);
            conn.update_station_auto(&result.station, "AUTO")?;
        }
    }

    let (_x, _y, inserted) = conn.insert_checks(checks)?;
    conn.insert_station_check_steps(&steps)?;
    conn.update_station_with_check_data(&inserted, true)?;

    if add_streaming_servers {
        let mut urls_full: Vec<_> = inserted
            .iter()
            .filter_map(|station| Url::parse(&station.url).ok())
            .map(|mut url| {
                url.set_path("/");
                url.set_query(None);
                url.set_fragment(None);
                url.to_string()
            })
            .collect();

        urls_full.sort();
        urls_full.dedup();

        conn.insert_streaming_servers(
            urls_full
                .drain(..)
                .map(|base_url| DbStreamingServerNew::new(base_url, None, None, None))
                .collect(),
        )?;
    }

    Ok(checked_count)
}
