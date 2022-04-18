use crate::check::diff_calc::DiffCalc;
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
    pub station: DiffCalc<DbStationItem>,
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
    diff: DiffCalc<DbStationItem>,
    source: &str,
    timeout: u64,
    max_depth: u8,
    retries: u8,
) -> StationOldNew {
    let checkuuid = Uuid::new_v4().to_hyphenated().to_string();
    let now = Instant::now();
    trace!("Check started: {} - {}", diff.new.stationuuid, diff.new.name);
    let checks: StreamCheckResult =
        av_stream_info_rust::check_tree(&diff.new.url, timeout as u32, max_depth, retries, true);
    let timing_ms = now.elapsed().as_millis();
    let (steps, check) = flatten_check_result(
        diff.new.stationuuid.clone(),
        checkuuid.clone(),
        checks,
        None,
        source,
        timing_ms,
    );
    trace!("Check finished: {} - {}", diff.new.stationuuid, diff.new.name);

    match check {
        Some(check) => StationOldNew {
            station: diff,
            check,
            steps,
        },
        None => {
            let check = StationCheckItemNew::broken(
                diff.new.stationuuid.clone(),
                checkuuid,
                source.to_string(),
                timing_ms,
            );
            StationOldNew {
                station: diff,
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
    let agent = format!("{}/{}", crate_name!(), crate_version!());

    let client = Client::builder()
        .user_agent(&agent)
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
            .map(|station| DiffCalc::new(station))
            .map(|mut diff| {
                // check current favicon
                if !diff.new.favicon.is_empty() && recheck_existing_favicon {
                    trace!(
                        "checking favicon {} '{}'",
                        diff.new.stationuuid,
                        diff.new.favicon
                    );
                    let request = client.head(&diff.new.favicon).send();
                    //let link = ImageLink::new(&diff.new.favicon, agent, timeout);
                    let remove = match request {
                        Ok(request) => {
                            let status = request.status();
                            status.is_client_error() || status.is_server_error()
                        },
                        Err(_) => true
                    };
                    if remove {
                        trace!(
                            "removed favicon {} '{}'",
                            diff.new.stationuuid, diff.new.favicon
                        );
                        // reset favicon, it could not be loaded
                        diff.new.set_favicon(String::new());
                    }
                }
                diff
            })
            .map(|mut diff| {
                if diff.new.favicon.is_empty() && enable_extract_favicon {
                    trace!("searching favicon {}", diff.new.stationuuid);
                    let links = ImageLink::from_website(&diff.new.homepage, &agent, timeout);
                    if let Ok(links) = links {
                        let icon = get_best_icon(
                            links,
                            favicon_size_optimum,
                            favicon_size_min,
                            favicon_size_max,
                        );
                        if let Some(icon) = icon {
                            diff.new.set_favicon(icon.url.to_string());
                            trace!(
                                "added favicon {} '{}'",
                                diff.new.stationuuid,
                                diff.new.favicon
                            );
                        }
                    }
                }
                diff
            })
            .map(|mut diff| {
                let lang_copy = diff.new.language.clone();
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
                diff.new.set_language(lang_trimmed.join(","));
                diff
            })
            .map(|mut diff| {
                let tags_copy = diff.new.tags.clone();
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
                diff.new.set_tags(tags_trimmed.join(","));
                diff
            })
            .map(|mut diff| {
                let language = diff.new.language.clone();
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
                let codes = diff.new.languagecodes.clone();
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
                diff.new.set_languagecodes(codes_trimmed.join(","));
                diff
            })
            .map(|mut diff| {
                diff.new.set_homepage(Url::parse(&diff.new.homepage).map(|u|u.to_string()).unwrap_or_default());
                diff.new.set_url(Url::parse(&diff.new.url).map(|u|u.to_string()).unwrap_or_default());
                diff
            })
            .map(|diff| dbcheck_internal(diff, source, timeout, max_depth, retries))
            .map(|mut diff| {
                if diff.check.metainfo_overrides_database {
                    debug!("override station: uuid='{}'", diff.station.new.stationuuid);
                    if let Some(name) = diff.check.name.clone() {
                        diff.station.new.name = name;
                    }
                    if let Some(homepage) = diff.check.homepage.clone() {
                        diff.station.new.homepage = homepage;
                    }
                    if let Some(loadbalancer) = diff.check.loadbalancer.clone() {
                        diff.station.new.url = loadbalancer;
                    }
                    if let Some(countrycode) = diff.check.countrycode.clone() {
                        diff.station.new.countrycode = countrycode;
                    }
                    if diff.check.countrysubdivisioncode.is_some() {
                        diff.station.new.iso_3166_2 = diff.check.countrysubdivisioncode.clone().map(|s|s.to_uppercase().to_string());
                    }
                    if let Some(tags) = diff.check.tags.clone() {
                        diff.station.new.tags = tags;
                    }
                    if let Some(favicon) = diff.check.favicon.clone() {
                        diff.station.new.favicon = favicon;
                    }
                    if diff.check.geo_lat.is_some() {
                        diff.station.new.geo_lat = diff.check.geo_lat.clone();
                    }
                    if diff.check.geo_long.is_some() {
                        diff.station.new.geo_long = diff.check.geo_long.clone();
                    }
                    if let Some(languagecodes) = diff.check.languagecodes.clone() {
                        diff.station.new.languagecodes = languagecodes;
                    }
                }
                diff
            })
            .collect()
    });

    // do real insert
    let mut checks = vec![];
    let mut steps = vec![];
    for result in results {
        checks.push(result.check);
        steps.extend(result.steps);

        if result.station.changed() {
            debug!("changed {}", result.station.new.stationuuid);
            conn.update_station_auto(&result.station.new, "AUTO")?;
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
