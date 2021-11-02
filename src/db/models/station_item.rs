use chrono::DateTime;
use chrono::Utc;

#[derive(Clone, Debug)]
pub struct DbStationItem {
    pub id: i32,
    pub changeuuid: String,
    pub stationuuid: String,
    pub serveruuid: Option<String>,
    pub name: String,
    pub url: String,
    pub url_resolved: String,
    pub homepage: String,
    pub favicon: String,
    pub tags: String,
    pub country: String,
    pub countrycode: String,
    pub iso_3166_2: Option<String>,
    pub state: String,
    pub language: String,
    pub languagecodes: String,
    pub votes: i32,
    pub lastchangetime: String,
    pub lastchangetime_iso8601: Option<DateTime<Utc>>,
    pub codec: String,
    pub bitrate: u32,
    pub hls: bool,
    pub lastcheckok: bool,
    pub lastchecktime: String,
    pub lastchecktime_iso8601: Option<DateTime<Utc>>,
    pub lastcheckoktime: String,
    pub lastcheckoktime_iso8601: Option<DateTime<Utc>>,
    pub lastlocalchecktime: String,
    pub lastlocalchecktime_iso8601: Option<DateTime<Utc>>,
    pub clicktimestamp: String,
    pub clicktimestamp_iso8601: Option<DateTime<Utc>>,
    pub clickcount: u32,
    pub clicktrend: i32,
    pub ssl_error: bool,
    pub geo_lat: Option<f64>,
    pub geo_long: Option<f64>,
    pub has_extended_info: Option<bool>,
    pub changed: bool,
}

impl DbStationItem {
    pub fn get_changed(&self) -> bool {
        self.changed
    }

    pub fn set_favicon<P: AsRef<str>>(&mut self, favicon: P) {
        if !self.favicon.eq(favicon.as_ref()) {
            debug!(
                "station changed {}: favicon '{}' -> '{}'",
                self.stationuuid,
                self.favicon,
                favicon.as_ref()
            );
            self.favicon = favicon.as_ref().to_string();
            self.changed = true;
        }
    }

    pub fn set_language<P: AsRef<str>>(&mut self, language: P) {
        if !self.language.eq(language.as_ref()) {
            debug!(
                "station changed {}: language '{}' -> '{}'",
                self.stationuuid,
                self.language,
                language.as_ref()
            );
            self.language = language.as_ref().to_string();
            self.changed = true;
        }
    }

    pub fn set_tags<P: AsRef<str>>(&mut self, tags: P) {
        if !self.tags.eq(tags.as_ref()) {
            debug!(
                "station changed {}: tags '{}' -> '{}'",
                self.stationuuid,
                self.tags,
                tags.as_ref()
            );
            self.tags = tags.as_ref().to_string();
            self.changed = true;
        }
    }

    pub fn set_languagecodes<P: AsRef<str>>(&mut self, languagecodes: P) {
        if !self.languagecodes.eq(languagecodes.as_ref()) {
            debug!(
                "station changed {}: languagecodes '{}' -> '{}'",
                self.stationuuid,
                self.languagecodes,
                languagecodes.as_ref()
            );
            self.languagecodes = languagecodes.as_ref().to_string();
            self.changed = true;
        }
    }

    pub fn set_url<P: AsRef<str>>(&mut self, url: P) {
        if !self.url.eq(url.as_ref()) {
            debug!(
                "station changed {}: url '{}' -> '{}'",
                self.stationuuid,
                self.url,
                url.as_ref()
            );
            self.url = url.as_ref().to_string();
            self.changed = true;
        }
    }

    pub fn set_homepage<P: AsRef<str>>(&mut self, homepage: P) {
        if !self.homepage.eq(homepage.as_ref()) {
            debug!(
                "station changed {}: homepage '{}' -> '{}'",
                self.stationuuid,
                self.homepage,
                homepage.as_ref()
            );
            self.homepage = homepage.as_ref().to_string();
            self.changed = true;
        }
    }
    /*
    pub fn set_last_check_ok(&mut self, check_ok: bool) {
        if self.lastcheckok != check_ok {
            self.lastcheckok = check_ok;
            self.changed = true;
        }
    }

    pub fn set_codec<P: AsRef<str>>(&mut self, codec: P) {
        if self.codec != codec.as_ref() {
            self.codec = codec.as_ref().to_string();
            self.changed = true;
        }
    }

    pub fn set_bitrate(&mut self, bitrate: u32) {
        if self.bitrate != bitrate {
            self.bitrate = bitrate;
            self.changed = true;
        }
    }

    pub fn set_hls(&mut self, hls: bool) {
        if self.hls != hls {
            self.hls = hls;
            self.changed = true;
        }
    }
    */
}
