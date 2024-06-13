// TODO
#![feature(trivial_bounds)]

mod error;
mod sys;

pub use crate::error::{Error, Result};

/// TODO: Document
///
/// When the manager is dropped, the handler is no longer guaranteed to receive
/// updates.
pub struct Manager {
    inner: sys::Manager,
}

impl Manager {
    pub fn new<T>(handler: T) -> Self
    where
        T: Handler,
    {
        Manager {
            inner: sys::Manager::new(handler),
        }
    }

    pub fn request_authorization(&self) {
        self.inner.request_authorization();
    }

    pub fn update_once(&self) {
        self.inner.update_once()
    }

    pub fn start_updates(&self) {
        self.inner.start_updates()
    }

    pub fn stop_updates(&self) {
        self.inner.stop_updates()
    }
}

pub trait Handler: 'static {
    fn handle(&self, location: Location<'_>);
    fn error(&self, error: Error);
}

pub struct Location<'a> {
    inner: sys::Location<'a>,
}

impl Location<'_> {
    pub fn coordinates(&self) -> Coordinates {
        self.inner.coordinates()
    }

    pub fn altitude(&self) -> f64 {
        self.inner.altitude()
    }

    /// The direction in which the device is travelling, measured in degrees and
    /// relative to due north.
    pub fn bearing(&self) -> f64 {
        self.inner.bearing()
    }

    /// The instantaneous speed of the device measured in meters per second.
    pub fn speed(&self) -> f64 {
        self.inner.speed()
    }

    pub fn time(&self) {
        self.inner.time()
    }

    // TODO: Accuracies
}

#[derive(Copy, Clone, Debug)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Copy, Clone, Debug)]
pub enum Accuracy {
    Approximate,
    Precise,
}

#[derive(Copy, Clone, Debug)]
pub enum Category {
    Foreground,
    Background,
}
