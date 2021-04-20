use crate::thread::{SendRecvError, ShutdownHandle, ShutdownHandlingThread};
use config::Config;
use crossbeam::channel::{self, Receiver, Sender, TryRecvError};
use err_derive::Error;
use map_tiler::{Config as MapTilerConfig, MapTiler};
use osm_client::{Daylight, OsmClient, Scale, Zoom};
use std::io;
use tiny_skia::Pixmap;

// https://docs.rs/crossbeam/0.8.0/crossbeam/channel/index.html
// TODO
// - traits and boxed stuff, client newtype over Sender
// - multiple recvrs, one for tiles, another for config stuff, use crossbeam select utils

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "IO error")]
    Io(#[error(source)] io::Error),

    #[error(display = "{}", _0)]
    MapTilerError(#[error(source)] map_tiler::Error),

    #[error(display = "{}", _0)]
    SendRecv(#[error(source)] SendRecvError),
}

#[derive(Debug)]
pub struct GetTilesRequest {
    pub lat: f64,
    pub lon: f64,
    pub zoom: Zoom,
}

// TODO image: Image, once it has Send
#[derive(Debug)]
pub struct GetTilesResponse {
    pub image: Pixmap,
}

// TODO MapTileServiceConfigClient or just tack on some Option fields in the request
#[derive(Debug, Clone)]
pub struct MapTileServiceClient {
    req_sender: Sender<GetTilesRequest>,
    resp_recvr: Receiver<GetTilesResponse>,
}

impl MapTileServiceClient {
    pub fn new(
        req_sender: Sender<GetTilesRequest>,
        resp_recvr: Receiver<GetTilesResponse>,
    ) -> Self {
        MapTileServiceClient {
            req_sender,
            resp_recvr,
        }
    }

    // TODO consider the try_send with timeout
    pub fn request(&self, lat: f64, lon: f64, zoom: Zoom) -> Result<(), Error> {
        log::debug!("Request tiles {}, {}, {}", lat, lon, zoom);
        self.req_sender
            .send(GetTilesRequest { lat, lon, zoom })
            .map_err(SendRecvError::from)?;
        Ok(())
    }

    pub fn try_recv(&self) -> Result<Option<Pixmap>, Error> {
        match self.resp_recvr.try_recv() {
            Ok(resp) => Ok(Some(resp.image)),
            Err(e) => match e {
                TryRecvError::Empty => Ok(None),
                TryRecvError::Disconnected => Err(SendRecvError::RecvChannelDisconnected.into()),
            },
        }
    }
}

#[derive(Debug)]
pub struct MapTileService {
    map_tiler: MapTiler,
    resp_sender: Sender<GetTilesResponse>,
}

impl MapTileService {
    fn new(config: Config, resp_sender: Sender<GetTilesResponse>) -> Result<Self, Error> {
        // TODO - use config
        let client = OsmClient::new(config.tiler.url)
            .with_daylight(Daylight::Day)
            .with_scale(Scale::Four);
        let map_tiler = MapTiler::new(
            client,
            MapTilerConfig {
                width: config.window.width.into(),
                height: config.window.height.into(),
                tile_size: 1024, // TODO - config
            },
        )?;
        Ok(MapTileService {
            map_tiler,
            resp_sender,
        })
    }

    pub fn start(config: Config) -> Result<(MapTileServiceClient, ShutdownHandle), Error> {
        let (tile_req_sender, tile_req_recvr) = channel::bounded(2);
        let (tile_resp_sender, tile_resp_recvr) = channel::bounded(2);
        let service = MapTileService::new(config, tile_resp_sender)?;
        let shutdown_handle = service.spawn("MapTileService".to_string(), tile_req_recvr)?;
        Ok((
            MapTileServiceClient::new(tile_req_sender, tile_resp_recvr),
            shutdown_handle,
        ))
    }

    fn process_tile_request(&mut self, req: GetTilesRequest) -> Result<GetTilesResponse, Error> {
        let image = self
            .map_tiler
            .request_tiles(req.lat, req.lon, req.zoom)?
            .clone();
        Ok(GetTilesResponse { image })
    }
}

impl ShutdownHandlingThread for MapTileService {
    type Msg = GetTilesRequest;
    type ShutdownError = Error;

    fn handle_requests(&mut self, requests: Vec<Self::Msg>) -> Result<(), Self::ShutdownError> {
        for req in requests.into_iter() {
            // TODO - consider revising this to not shutdown on all errors, some things are
            // tolerable
            // - server timeout/not up yet stuff is ok, just retry
            let resp = self.process_tile_request(req)?;
            self.resp_sender
                .send(resp)
                .map_err(|_| SendRecvError::SendChannelDisconnected)?;
        }
        Ok(())
    }
}
