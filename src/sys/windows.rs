use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use windows::{
    Devices::Geolocation::{
        GeolocationAccessStatus, Geolocator, Geoposition, PositionStatus, StatusChangedEventArgs,
    },
    Foundation::{EventRegistrationToken, TypedEventHandler},
};

use crate::{Coordinates, Error, Handler, Result};

// TODO: Remove status changed handler from geolocator when dropped?

pub(crate) struct Manager {
    inner: Arc<Geolocator>,
    handler: TypedEventHandler<Geolocator, StatusChangedEventArgs>,
    // TODO: Inefficient
    rust_handler: Arc<Mutex<dyn Handler>>,
    token: Option<EventRegistrationToken>,
}

impl Manager {
    pub fn new<T>(handler: T) -> Self
    where
        T: Handler,
    {
        let geolocator = Arc::new(Geolocator::new().unwrap());
        let rust_handler = Arc::new(Mutex::new(handler));
        let rust_handler_cloned = rust_handler.clone();

        let event_handler: TypedEventHandler<Geolocator, StatusChangedEventArgs> =
            TypedEventHandler::new(
                move |geolocator: &Option<Geolocator>, status: &Option<StatusChangedEventArgs>| {
                    match status.as_ref() {
                        Some(status) => match status.Status() {
                            Ok(status) => match status {
                                PositionStatus::Ready => {
                                    // TODO: unwrap?
                                    let geolocator = geolocator.as_ref().unwrap();
                                    rust_handler_cloned
                                        .lock()
                                        .unwrap()
                                        .handle(get_location(geolocator));
                                }
                                PositionStatus::Initializing => {}
                                PositionStatus::NoData => {}
                                PositionStatus::Disabled => {}
                                PositionStatus::NotInitialized => {}
                                PositionStatus::NotAvailable => {}
                                _ => todo!(),
                            },
                            Err(_) => todo!(),
                        },
                        // TODO
                        None => {
                            todo!();
                        }
                    }

                    Ok(())
                },
            );

        Self {
            inner: geolocator,
            handler: event_handler,
            rust_handler,
            token: None,
        }
    }

    pub fn request_authorization(&self) -> Result<()> {
        // TODO: Could do an async API, but like :shrug:. No other platform has async
        // and this should only be run once per program.
        match Geolocator::RequestAccessAsync().unwrap().get().unwrap() {
            GeolocationAccessStatus::Allowed => Ok(()),
            GeolocationAccessStatus::Denied => Err(Error::AuthorizationDenied),
            _ => Err(Error::Unknown),
        }
    }

    pub fn update_once(&self) {
        #[cfg(not(feature = "async"))]
        use std::thread::spawn;

        #[cfg(feature = "async")]
        use tokio::task::spawn_blocking as spawn;

        let handler = self.rust_handler.clone();
        let inner = SyncGeolocator(self.inner.clone());

        // TODO: This is a roundabout way to do this

        spawn(move || {
            handler
                .lock()
                .unwrap()
                .handle(get_location(inner.0.as_ref()))
        });
    }

    pub fn start_updates(&mut self) {
        let token = self.inner.StatusChanged(&self.handler).unwrap();
        self.token = Some(token);
    }

    pub fn stop_updates(&mut self) {
        if let Some(token) = self.token.take() {
            self.inner.RemoveStatusChanged(token).unwrap();
        }
    }
}

pub struct Location<'a> {
    inner: Geoposition,
    _phantom_data: PhantomData<&'a ()>,
}

// TODO: Civic and venue data?

impl Location<'_> {
    pub fn coordinates(&self) -> Coordinates {
        let coordinates = self.inner.Coordinate().unwrap();
        Coordinates {
            latitude: coordinates.Latitude().unwrap(),
            longitude: coordinates.Longitude().unwrap(),
        }
    }

    pub fn altitude(&self) -> f64 {
        unimplemented!();
    }

    pub fn bearing(&self) -> f64 {
        unimplemented!();
    }

    pub fn speed(&self) -> f64 {
        unimplemented!();
    }

    pub fn time(&self) {
        unimplemented!();
    }
}

fn get_location(geolocator: &Geolocator) -> crate::Location {
    crate::Location {
        inner: Location {
            inner: geolocator.GetGeopositionAsync().unwrap().get().unwrap(),
            _phantom_data: PhantomData,
        },
    }
}

struct SyncGeolocator(Arc<Geolocator>);

// TODO FIXME: This API is thread-safe right?
// TODO: Safety
unsafe impl Send for SyncGeolocator {}
unsafe impl Sync for SyncGeolocator {}
