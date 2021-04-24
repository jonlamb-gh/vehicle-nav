use common::{Latitude, Longitude, Zoom};
use config::Config;
use std::collections::BTreeMap;

/// Given a zoom level, return a delta lat/lon for a fixed distance shift.
/// Provides a constant change in pixel distance.
#[derive(Debug, Clone)]
pub struct ZoomDeltaMap {
    map: BTreeMap<Zoom, Delta>,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
struct Delta {
    lat: Latitude,
    lon: Longitude,
}

impl ZoomDeltaMap {
    pub fn new(config: &Config) -> Self {
        // TODO - do the transform stuff
        let mut map = BTreeMap::new();
        for z in Zoom::MIN.get()..Zoom::MAX.get() {
            let lat = Latitude(0.001);
            let lon = Longitude(0.001);
            map.insert(z.into(), Delta { lat, lon });
        }
        ZoomDeltaMap { map }
    }

    pub fn get(&self, zoom: Zoom) -> (Latitude, Longitude) {
        let d = self.map.get(&zoom).expect("The ZoomDeltaMap is messed up");
        (d.lat, d.lon)
    }
}
