use crate::db::models;
use crate::db::models::StationCheckItemNew;
use crate::db::models::StationCheckStepItemNew;
use crate::db::models::StationItem;
use av_stream_info_rust::StreamCheckResult;
use std::time::Instant;
use uuid::Uuid;

use rayon::prelude::*;

use av_stream_info_rust;
use av_stream_info_rust::UrlType;

use std;

use crate::db::connect;

use colored::*;

#[derive(Clone, Debug)]
pub struct StationOldNew {
    pub station: StationItem,
    pub check: StationCheckItemNew,
    pub steps: Vec<StationCheckStepItemNew>,
}

fn check_for_change(
    old: &models::StationItem,
    new: &StationCheckItemNew,
    timing_ms: u128,
) -> (bool, String) {
    let mut retval = false;
    let mut result = String::from("");

    if old.lastcheckok != new.check_ok {
        if new.check_ok {
            result.push('+');
            result.red();
        } else {
            result.push('-');
        }
        retval = true;
    } else {
        result.push('~');
    }
    result.push(' ');
    result.push('\'');
    result.push_str(&old.name);
    result.push('\'');
    result.push(' ');
    result.push_str(&old.url);
    if old.hls != new.hls {
        result.push_str(&format!(" hls:{}->{}", old.hls, new.hls));
        retval = true;
    }
    if old.bitrate != new.bitrate {
        result.push_str(&format!(" bitrate:{}->{}", old.bitrate, new.bitrate));
        retval = true;
    }
    if old.codec != new.codec {
        result.push_str(&format!(" codec:{}->{}", old.codec, new.codec));
        retval = true;
    }
    result.push_str(&format!(" ({}ms)", timing_ms));
    if old.lastcheckok != new.check_ok {
        if new.check_ok {
            return (retval, result.green().to_string());
        } else {
            return (retval, result.red().to_string());
        }
    } else {
        return (retval, result.yellow().to_string());
    }
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
    station: StationItem,
    source: &str,
    timeout: u64,
    max_depth: u8,
    retries: u8,
) -> StationOldNew {
    let checkuuid = Uuid::new_v4().to_hyphenated().to_string();
    let now = Instant::now();
    trace!("Check started: {} - {}", station.stationuuid, station.name);
    let checks =
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

pub fn dbcheck(
    connection_str: String,
    source: &str,
    concurrency: usize,
    stations_count: u32,
    timeout: u64,
    max_depth: u8,
    retries: u8,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut conn = connect(connection_str)?;
    let stations = conn.get_stations_to_check(24, stations_count)?;
    let checked_count = stations.len();

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build()?;
    let results: Vec<_> = pool.install(|| {
        stations
            .into_par_iter()
            .map(|station| dbcheck_internal(station, source, timeout, max_depth, retries))
            .collect()
    });
    for result in results.iter() {
        let timing_ms = result.check.timing_ms;
        let (changed, change_str) = check_for_change(&result.station, &result.check, timing_ms);
        if changed {
            debug!("{}", change_str.red());
        } else {
            debug!("{}", change_str.dimmed());
        }
    }

    // do real insert
    let mut checks = vec![];
    let mut steps = vec![];
    for result in results {
        checks.push(result.check);
        steps.extend(result.steps);
    }

    let (_x, _y, inserted) = conn.insert_checks(checks)?;
    conn.insert_station_check_steps(&steps)?;
    conn.update_station_with_check_data(&inserted, true)?;

    Ok(checked_count)
}
