use std::marker::PhantomData;

use windows::{
    Devices::Geolocation::{Geolocator, Geoposition, PositionStatus, StatusChangedEventArgs},
    Foundation::TypedEventHandler,
};

use crate::{Coordinates, Handler};

// TODO: Remove status changed handler from geolocator when dropped?

pub(crate) struct Manager {
    inner: Geolocator,
    _handler: TypedEventHandler<Geolocator, StatusChangedEventArgs>,
}

impl Manager {
    pub fn new<T>(handler: T) -> Self
    where
        T: Handler,
    {
        let geolocator = Geolocator::new().unwrap();
        let event_handler: TypedEventHandler<Geolocator, StatusChangedEventArgs> =
            TypedEventHandler::new(
                move |geolocator: &Option<Geolocator>, status: &Option<StatusChangedEventArgs>| {
                    println!("called event handler");
                    match status.as_ref() {
                        Some(status) => match status.Status() {
                            Ok(status) => match status {
                                PositionStatus::Ready => {
                                    // TODO: unwrap?
                                    let geolocator = geolocator.as_ref().unwrap();
                                    let location = crate::Location {
                                        inner: Location {
                                            inner: geolocator
                                                .GetGeopositionAsync()
                                                .unwrap()
                                                .get()
                                                .unwrap(),
                                            _phantom_data: PhantomData,
                                        },
                                    };

                                    handler.handle(location);
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
                        None => {}
                    }

                    Ok(())
                },
            );
        geolocator.StatusChanged(&event_handler).unwrap();

        Self {
            inner: geolocator,
            _handler: event_handler,
        }
    }

    pub fn request_authorization(&self) {
        let temp = Geolocator::RequestAccessAsync().unwrap().get().unwrap();
        println!("temp: {temp:#?}");
    }

    pub fn update_once(&self) {
        // TODO: Do we want to
    }

    pub fn start_updates(&self) {}

    pub fn stop_updates(&self) {}
}

pub struct Location<'a> {
    inner: Geoposition,
    _phantom_data: PhantomData<&'a ()>,
}

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
