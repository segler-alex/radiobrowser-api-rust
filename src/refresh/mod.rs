use std;
use std::collections::HashMap;
use crate::thread;
use crate::db::connect;
use crate::db::DbConnection;

pub struct RefreshCacheStatus{
    old_items: usize,
    new_items: usize,
    changed_items: usize,
}

pub fn refresh_cache_items(
    pool: &Box<dyn DbConnection>,
    cache_table_name: &str,
    cache_column_name: &str,
    station_column_name: &str,
)-> Result<RefreshCacheStatus, Box<dyn std::error::Error>> {
    let items_cached = pool.get_cached_items(cache_table_name, cache_column_name)?;
    let items_current = pool.get_stations_multi_items(station_column_name)?;
    let mut changed = 0;
    let max_cache_item_len = 110;

    let mut to_delete = vec![];
    for item_cached in items_cached.keys() {
        if !items_current.contains_key(item_cached) {
            to_delete.push(item_cached);
        }
    }
    pool.remove_from_cache(to_delete, cache_table_name, cache_column_name)?;

    let mut to_insert: HashMap<&String, (u32,u32)> = HashMap::new();
    for item_current in items_current.keys() {
        if !items_cached.contains_key(item_current) {
            if item_current.len() < max_cache_item_len {
                to_insert.insert(item_current, *items_current.get(item_current).unwrap_or(&(0,0)));
            }else{
                warn!("cached '{}' item too long: '{}'", station_column_name, item_current);
            }
        } else {
            let value_new = *items_current.get(item_current).unwrap_or(&(0,0));
            let value_old = *items_cached.get(item_current).unwrap_or(&(0,0));
            if value_old != value_new {
                pool.update_cache_item(
                    item_current,
                    value_new.0,
                    value_new.1,
                    cache_table_name,
                    cache_column_name,
                )?;
                changed = changed + 1;
            }
        }
    }
    pool.insert_to_cache(to_insert, cache_table_name, cache_column_name)?;
    trace!(
        "{}: {} -> {}, Changed: {}",
        station_column_name,
        items_cached.len(),
        items_current.len(),
        changed
    );
    Ok(
    RefreshCacheStatus{
        old_items: items_cached.len(),
        new_items: items_current.len(),
        changed_items: changed,
    })
}

fn refresh_worker(connection_string: String) -> Result<(), Box<dyn std::error::Error>> {
    let pool = connect(connection_string)?;
    trace!("REFRESH START");
    let tags = refresh_cache_items(&pool, "TagCache", "TagName", "Tags")?;
    let languages = refresh_cache_items(&pool, "LanguageCache", "LanguageName", "Language")?;
    debug!("Refresh(Tags={}->{} changed={}, Languages={}->{} changed={})", tags.old_items, tags.new_items, tags.changed_items, languages.old_items, languages.new_items, languages.changed_items);
    Ok(())
}

pub fn start(connection_string: String, update_caches_interval: u64) {
    if update_caches_interval > 0 {
        thread::spawn(move || {
            loop {
                let result = refresh_worker(connection_string.clone());
                match result {
                    Ok(_)=>{
                    },
                    Err(err)=>{
                        error!("Refresh worker error: {}", err);
                    }
                }
                thread::sleep(::std::time::Duration::new(update_caches_interval, 0));
            }
        });
    }
}