use thread;
use threadpool::ThreadPool;

use av_stream_info_rust;
use check::favicon;

use std;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use check::models;
use check::models::StationCheckItemNew;

use check::db;
use mysql;

use colored::*;

fn check_for_change(
    old: &models::StationItem,
    new: &StationCheckItemNew,
    new_favicon: &str,
) -> (bool, String) {
    let mut retval = false;
    let mut result = String::from("");

    if old.check_ok != new.check_ok {
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
    }*/
    if old.favicon != new_favicon {
        result.push_str(&format!(" favicon: {} -> {}", old.favicon, new_favicon));
        retval = true;
    }
    if old.check_ok != new.check_ok {
        if new.check_ok {
            return (retval, result.green().to_string());
        } else {
            return (retval, result.red().to_string());
        }
    } else {
        return (retval, result.yellow().to_string());
    }
}

fn update_station(
    conn: &mysql::Pool,
    old: &models::StationItem,
    new_item: &StationCheckItemNew,
    new_favicon: &str,
) {
    let result = db::insert_check(&conn, &new_item);
    if let Err(err) = result {
        debug!("Insert check error {}", err);
    }
    db::update_station(&conn, &new_item);
    let (changed, change_str) = check_for_change(&old, &new_item, new_favicon);
    if changed {
        debug!("{}", change_str.red());
    } else {
        debug!("{}", change_str.dimmed());
    }
}

pub fn dbcheck(
    connection_str: &str,
    source: &str,
    concurrency: usize,
    stations_count: u32,
    useragent: &str,
    timeout: u32,
    max_depth: u8,
    retries: u8,
    favicon_checks: bool,
) -> u32 {
    let conn = db::new(connection_str);
    let mut checked_count = 0;
    match conn {
        Ok(conn) => {
            let stations = db::get_stations_to_check(&conn, 24, stations_count);

            let pool = ThreadPool::new(concurrency);
            for station in stations {
                checked_count = checked_count + 1;
                let source = String::from(source);
                let useragent = String::from(useragent);
                let conn = conn.clone();
                pool.execute(move || {
                    let (_, receiver): (Sender<i32>, Receiver<i32>) = channel();
                    let station_name = station.name.clone();
                    let max_timeout = (retries as u32) * timeout * 2;
                    thread::spawn(move || {
                        for _ in 0..max_timeout {
                            thread::sleep(Duration::from_secs(1));
                            let o = receiver.try_recv();
                            match o {
                                Ok(_) => {
                                    return;
                                }
                                Err(value) => match value {
                                    TryRecvError::Empty => {}
                                    TryRecvError::Disconnected => {
                                        return;
                                    }
                                },
                            }
                        }
                        debug!("Still not finished: {}", station_name);
                        std::process::exit(0x0100);
                    });
                    let mut new_item: StationCheckItemNew = StationCheckItemNew {
                        station_uuid: station.uuid.clone(),
                        source: source.clone(),
                        codec: "".to_string(),
                        bitrate: 0,
                        hls: false,
                        check_ok: false,
                        url: "".to_string(),
                    };
                    let items =
                        av_stream_info_rust::check(&station.url, timeout, max_depth, retries);
                    for item in items.iter() {
                        match item {
                            &Ok(ref item) => {
                                let mut codec = item.CodecAudio.clone();
                                if let Some(ref video) = item.CodecVideo {
                                    codec.push_str(",");
                                    codec.push_str(&video);
                                }
                                new_item = StationCheckItemNew {
                                    station_uuid: station.uuid.clone(),
                                    source: source.clone(),
                                    codec: codec,
                                    bitrate: item.Bitrate as i32,
                                    hls: item.Hls,
                                    check_ok: true,
                                    url: item.Url.clone(),
                                };
                            }
                            &Err(_) => {}
                        }
                    }
                    if favicon_checks {
                        let new_favicon = favicon::check(
                            &station.homepage,
                            &station.favicon,
                            &useragent,
                            timeout,
                        );
                        update_station(&conn, &station, &new_item, &new_favicon);
                    } else {
                        update_station(&conn, &station, &new_item, &station.favicon);
                    }
                });
            }
            pool.join();
        }
        Err(e) => {
            debug!("Database connection error {}", e);
        }
    }
    checked_count
}
