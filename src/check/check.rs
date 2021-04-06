use std::time::Instant;
use crate::db::models;
use crate::db::models::StationItem;
use crate::db::models::StationCheckItemNew;

use rayon::prelude::*;

use av_stream_info_rust;

use std;

use crate::db::connect;

use colored::*;

#[derive(Clone,Debug)]
pub struct StationOldNew {
    pub station: StationItem,
    pub check: StationCheckItemNew,
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
    /*if old.urlcache != new.url{
        debug!("  url      :{}->{}",old.urlcache,new.url);
        retval = true;
    }
    if old.favicon != new_favicon {
        result.push_str(&format!(" favicon: {} -> {}", old.favicon, new_favicon));
        retval = true;
    }
    */
    result.push_str(&format!(" ({}ms)",timing_ms));
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

fn dbcheck_internal(
    station: StationItem,
    source: &str,
    timeout: u64,
    max_depth: u8,
    retries: u8,
) -> StationOldNew {
    let source = String::from(source);
    let now = Instant::now();
    let mut items = av_stream_info_rust::check(&station.url, timeout as u32, max_depth, retries);
    let timing_ms = now.elapsed().as_millis();
    for item in items.drain(..) {
        match item {
            Ok(item) => {
                let override_metadata = item.OverrideIndexMetaData.unwrap_or(false);
                let do_not_index = item.DoNotIndex.unwrap_or(false);
                if do_not_index && override_metadata {
                    // ignore non public streams
                    debug!("Ignore private stream: {} - {}", station.stationuuid, item.Url);
                }else{
                    let mut codec = item.CodecAudio.clone();
                    if let Some(ref video) = item.CodecVideo {
                        codec.push_str(",");
                        codec.push_str(&video);
                    }
                    let check = StationCheckItemNew {
                        checkuuid: None,
                        station_uuid: station.stationuuid.clone(),
                        source: source.clone(),
                        codec: codec,
                        bitrate: item.Bitrate.unwrap_or(0),
                        sampling: item.Sampling,
                        hls: item.Hls,
                        check_ok: true,
                        url: item.Url.clone(),
                        timestamp: None,

                        metainfo_overrides_database: override_metadata,
                        public: item.Public,
                        name: item.Name,
                        description: item.Description,
                        tags: item.Genre,
                        countrycode: item.CountryCode,
                        countrysubdivisioncode: item.CountrySubdivisonCode,
                        languagecodes: item.LanguageCodes,
                        homepage: item.Homepage,
                        favicon: item.LogoUrl,
                        loadbalancer: item.MainStreamUrl,
                        do_not_index: item.DoNotIndex,
                        timing_ms,
                        server_software: item.Server,
                    };
                    return StationOldNew {
                        station,
                        check,
                    };
                }
            }
            Err(_) => {}
        }
    }
    let check: StationCheckItemNew = StationCheckItemNew {
        checkuuid: None,
        station_uuid: station.stationuuid.clone(),
        source: source.clone(),
        codec: "".to_string(),
        bitrate: 0,
        sampling: None,
        hls: false,
        check_ok: false,
        url: "".to_string(),
        timestamp: None,
        
        metainfo_overrides_database: false,
        public: None,
        name: None,
        description: None,
        tags: None,
        countrycode: None,
        countrysubdivisioncode: None,
        languagecodes: vec![],
        homepage: None,
        favicon: None,
        loadbalancer: None,
        do_not_index: None,
        timing_ms,
        server_software: None,
    };
    return StationOldNew {
        station,
        check,
    };
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

    let pool = rayon::ThreadPoolBuilder::new().num_threads(concurrency).build()?;
    let mut results: Vec<_> = pool.install(||{
        stations.into_par_iter().map(|station| 
            dbcheck_internal(station, source, timeout, max_depth, retries)
        ).collect()
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
    let checks: Vec<_> = results.drain(..).map(|x|x.check).collect();
    let (_x,_y,inserted) = conn.insert_checks(checks)?;
    conn.update_station_with_check_data(&inserted, true)?;

    Ok(checked_count)
}
