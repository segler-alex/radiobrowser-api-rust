use crate::db::connect;
use crate::db::models::DbStreamingServer;
use icecast_stats::generate_icecast_stats_url;
use icecast_stats::IcecastStatsRoot;
use rayon::prelude::*;
use reqwest::blocking::get;
use url::Url;

fn single_check(server: &mut DbStreamingServer) -> Result<(), String> {
    let u = Url::parse(&server.url).or(Err(String::from("URLParseError")))?;
    let u = generate_icecast_stats_url(u);
    let result = get(u.clone()).or(Err(String::from("FetchError")))?;
    let t:IcecastStatsRoot = result.json().or(Err(String::from("ResultDecodeError")))?;
    let j = serde_json::to_string(&t).or(Err(String::from("ResultToJsonError")))?;
    server.status = Some(j);
    server.statusurl = Some(u.to_string());
    Ok(())
}

pub fn do_check(
    database_url: String,
    chunksize: u32,
    concurrency: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("do_check()");
    let mut conn_new_style = connect(database_url)?;
    let servers: Vec<DbStreamingServer> = conn_new_style.get_servers_to_check(24, chunksize)?;

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build()?;
    let updated_streaming_servers: Vec<_> = pool.install(|| {
        servers
            .into_par_iter()
            .map(|mut server| {
                trace!("checking {}", server.url);
                server.status = None;
                server.statusurl = None;
                match single_check(&mut server) {
                    Ok(_) => {
                        debug!("found icecast url at {}", server.url);
                    },
                    Err(err) => {
                        trace!("{}: {}", err, server.url);
                        server.error = Some(err);
                    }
                };
                server
            })
            .collect()
    });

    conn_new_style.update_streaming_servers(updated_streaming_servers)?;
    trace!("do_check() finished");

    Ok(())
}
