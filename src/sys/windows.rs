use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use windows::{
    Devices::Geolocation::{
        Geocoordinate, GeolocationAccessStatus, Geolocator, PositionStatus, StatusChangedEventArgs,
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
    pub fn new<T>(handler: T) -> Result<Self>
    where
        T: Handler,
    {
        let geolocator = Arc::new(Geolocator::new()?);
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
                                    if let Ok(handler) = rust_handler_cloned.lock() {
                                        if let Ok(location) = get_location(geolocator) {
                                            handler.handle(location)
                                        }
                                    }
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

        Ok(Self {
            inner: geolocator,
            handler: event_handler,
            rust_handler,
            token: None,
        })
    }

    pub fn request_authorization(&self) -> Result<()> {
        // TODO: Could do an async API, but like :shrug:. No other platform has async
        // and this should only be run once per program.
        match Geolocator::RequestAccessAsync()?.get()? {
            GeolocationAccessStatus::Allowed => Ok(()),
            GeolocationAccessStatus::Denied => Err(Error::AuthorizationDenied),
            _ => Err(Error::Unknown),
        }
    }

    pub fn update_once(&self) -> Result<()> {
        #[cfg(not(feature = "async"))]
        use std::thread::spawn;

        #[cfg(feature = "async")]
        use tokio::task::spawn_blocking as spawn;

        let handler = self.rust_handler.clone();
        let inner = SyncGeolocator(self.inner.clone());

        spawn(move || {
            if let Ok(handler) = handler.lock() {
                if let Ok(location) = get_location(inner.0.as_ref()) {
                    handler.handle(location)
                }
            }
        });

        Ok(())
    }

    pub fn start_updates(&mut self) -> Result<()> {
        let token = self.inner.StatusChanged(&self.handler)?;
        self.token = Some(token);
        Ok(())
    }

    pub fn stop_updates(&mut self) -> Result<()> {
        if let Some(token) = self.token.take() {
            self.inner.RemoveStatusChanged(token)?;
        }
        Ok(())
    }
}

pub struct Location<'a> {
    inner: Geocoordinate,
    _phantom_data: PhantomData<&'a ()>,
}

impl Location<'_> {
    pub fn coordinates(&self) -> Result<Coordinates> {
        Ok(Coordinates {
            latitude: self.inner.Latitude()?,
            longitude: self.inner.Longitude()?,
        })
    }

    pub fn altitude(&self) -> Result<f64> {
        self.inner.Altitude()?.Value().map_err(|e| e.into())
    }

    pub fn bearing(&self) -> Result<f64> {
        self.inner.Heading()?.Value().map_err(|e| e.into())
    }

    pub fn speed(&self) -> Result<f64> {
        self.inner.Speed()?.Value().map_err(|e| e.into())
    }

    pub fn time(&self) {
        unimplemented!();
    }
}

fn get_location(geolocator: &Geolocator) -> Result<crate::Location> {
    Ok(crate::Location {
        inner: Location {
            inner: geolocator.GetGeopositionAsync()?.get()?.Coordinate()?,
            _phantom_data: PhantomData,
        },
    })
}

struct SyncGeolocator(Arc<Geolocator>);

// TODO FIXME: This API is thread-safe right?
// TODO: Safety
unsafe impl Send for SyncGeolocator {}
unsafe impl Sync for SyncGeolocator {}

impl From<windows::core::Error> for Error {
    fn from(_: windows::core::Error) -> Self {
        Error::Unknown
    }
}
