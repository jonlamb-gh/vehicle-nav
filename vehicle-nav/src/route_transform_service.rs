use crate::thread::{SendRecvError, ShutdownHandle, ShutdownHandlingThread};
use common::{Coordinate, CoordinateTransform, Zoom};
use config::Config;
use crossbeam::channel::{self, Receiver, Sender, TryRecvError};
use err_derive::Error;
use raylib::ffi;
use std::io;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "IO error")]
    Io(#[error(source)] io::Error),

    #[error(display = "{}", _0)]
    SendRecv(#[error(source)] SendRecvError),
}

#[derive(Debug)]
pub enum Request {
    AddRouteCoordinate(AddRouteCoordinateRequest),
    GetRoute(GetRouteRequest),
}

#[derive(Debug)]
pub struct AddRouteCoordinateRequest {
    // RouteId
    coord: Coordinate,
}

#[derive(Debug)]
pub struct GetRouteRequest {
    // TODO - RouteId type
    // For now, there is only a single route
    map_center: Coordinate,
    map_zoom: Zoom,
}

#[derive(Debug)]
pub struct GetRouteResponse {
    // TODO - RouteId type
    // probably don't need to provide offset, but nice to have
    pub offset: usize,
    pub route_chunk: Vec<ffi::Vector2>, // TODO - config, chunk_size
}

#[derive(Debug, Clone)]
pub struct RouteTransformServiceClient {
    req_sender: Sender<Request>,
    resp_recvr: Receiver<GetRouteResponse>,
}

impl RouteTransformServiceClient {
    fn new(req_sender: Sender<Request>, resp_recvr: Receiver<GetRouteResponse>) -> Self {
        RouteTransformServiceClient {
            req_sender,
            resp_recvr,
        }
    }

    pub fn push_coordinate(&self, coord: Coordinate) -> Result<(), Error> {
        let req = Request::AddRouteCoordinate(AddRouteCoordinateRequest { coord });
        self.req_sender.send(req).map_err(SendRecvError::from)?;
        Ok(())
    }

    pub fn get_route(&self, map_center: Coordinate, map_zoom: Zoom) -> Result<(), Error> {
        log::debug!("Request route center {}, zoom {}", map_center, map_zoom);
        let req = Request::GetRoute(GetRouteRequest {
            map_center,
            map_zoom,
        });
        self.req_sender.send(req).map_err(SendRecvError::from)?;
        Ok(())
    }

    pub fn try_recv(&self) -> Result<Option<GetRouteResponse>, Error> {
        match self.resp_recvr.try_recv() {
            Ok(resp) => Ok(Some(resp)),
            Err(e) => match e {
                TryRecvError::Empty => Ok(None),
                TryRecvError::Disconnected => Err(SendRecvError::RecvChannelDisconnected.into()),
            },
        }
    }
}

#[derive(Debug)]
pub struct RouteTransformService {
    transform: CoordinateTransform,
    route: Vec<Coordinate>, // TODO ring buffer instead of vec
    resp_sender: Sender<GetRouteResponse>,
}

impl RouteTransformService {
    fn new(config: Config, resp_sender: Sender<GetRouteResponse>) -> Result<Self, Error> {
        let center_coord = Coordinate::from((
            config.startup_defaults.latitude,
            config.startup_defaults.longitude,
        ));
        let transform = CoordinateTransform::new(
            &center_coord,
            config.tiler.scale.unwrap_or_default(),
            config.startup_defaults.zoom,
            config.window.width.into(),
            config.window.height.into(),
        );
        Ok(RouteTransformService {
            transform,
            route: Vec::with_capacity(256), // TODO - config
            resp_sender,
        })
    }

    pub fn start(config: Config) -> Result<(RouteTransformServiceClient, ShutdownHandle), Error> {
        let (req_sender, req_recvr) = channel::bounded(32);
        let (route_resp_sender, route_resp_recvr) = channel::unbounded();
        let service = RouteTransformService::new(config, route_resp_sender)?;
        let shutdown_handle = service.spawn("RouteTransformService".to_string(), req_recvr)?;
        Ok((
            RouteTransformServiceClient::new(req_sender, route_resp_recvr),
            shutdown_handle,
        ))
    }

    fn process_new_coordinate(&mut self, req: AddRouteCoordinateRequest) -> Result<(), Error> {
        // TODO - manage the ring buffer
        self.route.push(req.coord);
        Ok(())
    }

    fn process_route_request(&mut self, req: GetRouteRequest) -> Result<GetRouteResponse, Error> {
        self.transform.update(&req.map_center, req.map_zoom);
        let route_chunk = self
            .route
            .iter()
            .map(|c| {
                let (x, y) = self.transform.coordinate_to_pixel(c);
                ffi::Vector2 {
                    x: x as _,
                    y: y as _,
                }
            })
            .collect();
        Ok(GetRouteResponse {
            offset: 0,
            route_chunk,
        })
    }
}

impl ShutdownHandlingThread for RouteTransformService {
    type Msg = Request;
    // TODO - just use String type once tolerable error cases are figured out
    type ShutdownError = Error;

    fn handle_requests(&mut self, requests: Vec<Self::Msg>) -> Result<(), Self::ShutdownError> {
        for req in requests.into_iter() {
            // TODO - consider revising this to not shutdown on all errors, some things are
            // tolerable
            // - put a result in the response
            match req {
                Request::AddRouteCoordinate(r) => self.process_new_coordinate(r)?,
                Request::GetRoute(r) => {
                    let resp = self.process_route_request(r)?;
                    self.resp_sender
                        .send(resp)
                        .map_err(|_| SendRecvError::SendChannelDisconnected)?;
                }
            }
        }
        Ok(())
    }
}
