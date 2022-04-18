use crate::db::models::StationHistoryItem;
use crate::db::DbConnection;
use crate::db::MysqlConnection;
use std::error::Error;

fn change_duplicated(item1: &StationHistoryItem, item2: &StationHistoryItem) -> bool {
    if item1.countrycode.eq(&item2.countrycode)
        && item1.favicon.eq(&item2.favicon)
        && item1.geo_lat.eq(&item2.geo_lat)
        && item1.geo_long.eq(&item2.geo_long)
        && item1.homepage.eq(&item2.homepage)
        && item1.language.eq(&item2.language)
        && item1.languagecodes.eq(&item2.languagecodes)
        && item1.name.eq(&item2.name)
        && item1.state.eq(&item2.state)
        && item1.tags.eq(&item2.tags)
        && item1.url.eq(&item2.url)
    {
        return true;
    }
    false
}

pub fn delete_duplicate_changes(
    conn: &mut MysqlConnection,
    min_change_count: u32,
) -> Result<(), Box<dyn Error>> {
    debug!("delete_duplicate_changes({})", min_change_count);
    let station_uuids = conn.get_stations_uuid_order_by_changes(min_change_count)?;
    let mut change_uuid_to_delete = vec![];
    let mut change_count = 0;
    let mut station_uuid_no = 0;
    let stations_uuids_count = station_uuids.len();
    for station_uuid in station_uuids {
        station_uuid_no += 1;
        let changes = conn.get_changes(Some(station_uuid.clone()), None, 1000)?;
        //let mut duplicates_current_change = 0;
        change_count += changes.len();
        if changes.len() > 0 {
            let mut current = &changes[0];
            for change_no in 1..changes.len() {
                if change_duplicated(current, &changes[change_no]) {
                    //trace!("duplicate found");
                    //duplicates_current_change += 1;
                    change_uuid_to_delete.push(changes[change_no].changeuuid.clone());
                } else {
                    current = &changes[change_no];
                }
                print!(
                    "\rduplication check: {}/{} {:03}/{:03} {:06}/{:08}",
                    station_uuid_no,
                    stations_uuids_count,
                    change_no,
                    changes.len(),
                    change_uuid_to_delete.len(),
                    change_count
                );
            }
        }
        /*
        if duplicates_current_change > 0{
            debug!("duplicates for {} found: {}/{}", station_uuid, duplicates_current_change, changes.len());
        }
        */
    }
    println!();
    info!(
        "duplicates found: {}/{}",
        change_uuid_to_delete.len(),
        change_count
    );
    conn.delete_change_by_uuid(&change_uuid_to_delete)?;
    info!("duplicates deleted: {}", change_uuid_to_delete.len());
    Ok(())
}

pub fn resethistory(conn: &mut MysqlConnection) -> Result<(), Box<dyn Error>> {
    debug!("resethistory()");
    conn.resethistory()?;
    println!("");
    Ok(())
}
