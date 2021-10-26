#[derive(Clone, Debug)]
pub struct StationCheckItemNew {
    pub checkuuid: Option<String>,
    pub station_uuid: String,
    pub source: String,
    pub codec: String,
    pub bitrate: u32,
    pub sampling: Option<u32>,
    pub hls: bool,
    pub check_ok: bool,
    pub url: String,
    pub timestamp: Option<String>,

    pub metainfo_overrides_database: bool,
    pub public: Option<bool>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub countrycode: Option<String>,
    pub countrysubdivisioncode: Option<String>,
    pub languagecodes: Option<String>,
    pub homepage: Option<String>,
    pub favicon: Option<String>,
    pub loadbalancer: Option<String>,
    pub do_not_index: Option<bool>,
    pub timing_ms: u128,
    pub server_software: Option<String>,
    pub ssl_error: bool,
    pub geo_lat: Option<f64>,
    pub geo_long: Option<f64>,
}

impl StationCheckItemNew {
    pub fn broken(station_uuid: String, check_uuid: String, source: String, timing_ms: u128) -> Self {
        StationCheckItemNew {
            checkuuid: Some(check_uuid),
            station_uuid,
            source,
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
            languagecodes: None,
            homepage: None,
            favicon: None,
            loadbalancer: None,
            do_not_index: None,
            timing_ms,
            server_software: None,
            ssl_error: false,
            geo_lat: None,
            geo_long: None,
        }
    }

    pub fn working(
        station_uuid: String,
        check_uuid: String,
        source: String,
        timing_ms: u128,
        url: String,
        info: av_stream_info_rust::StreamInfo,
    ) -> Self {
        let mut codec = info.CodecAudio.clone();
        if let Some(ref video) = info.CodecVideo {
            codec.push_str(",");
            codec.push_str(&video);
        }
        let latlong = info.GeoLatLong.clone().transpose().unwrap_or(None);
        StationCheckItemNew {
            checkuuid: Some(check_uuid),
            station_uuid,
            source,
            codec: codec,
            bitrate: info.Bitrate.unwrap_or(0),
            sampling: info.Sampling,
            hls: info.Hls,
            check_ok: true,
            url,
            timestamp: None,

            metainfo_overrides_database: info.OverrideIndexMetaData.unwrap_or(false),
            public: info.Public,
            name: info.Name,
            description: info.Description,
            tags: info.Genre,
            countrycode: info.CountryCode,
            countrysubdivisioncode: info.CountrySubdivisonCode,
            languagecodes: Some(info.LanguageCodes.join(",")),
            homepage: info.Homepage,
            favicon: info.LogoUrl,
            loadbalancer: info.MainStreamUrl,
            do_not_index: info.DoNotIndex,
            timing_ms,
            server_software: info.Server,
            ssl_error: info.SslError,
            geo_lat: latlong.clone().map(|y|y.lat),
            geo_long: latlong.map(|y|y.long),
        }
    }
}
