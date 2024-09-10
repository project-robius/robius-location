//! A library to access system location data.

mod error;
mod sys;

use std::time::SystemTime;

pub use crate::error::{Error, Result};

/// A manager for the location
///
/// As soon as the handler is registered it may receive updates, even if
/// `update_once` or `start_updates` aren't called. When the manager is dropped,
/// the handler will no longer guaranteed to receive updates.
pub struct Manager {
    inner: sys::Manager,
}

impl Manager {
    pub fn new<T>(handler: T) -> Result<Self>
    where
        T: Handler,
    {
        Ok(Manager {
            inner: sys::Manager::new(handler)?,
        })
    }

    /// Requests authorization to access location data.
    ///
    /// This will return immediately and request authorization in the background.
    pub fn request_authorization(&self, access: Access, accuracy: Accuracy) -> Result<()> {
        self.inner.request_authorization(access, accuracy)
    }

    /// Delivers a single update to the handler.
    pub fn update_once(&self) -> Result<()> {
        self.inner.update_once()
    }

    /// Begins delivering continuous updates to the handler.
    pub fn start_updates(&mut self) -> Result<()> {
        self.inner.start_updates()
    }

    /// Stops delivering continuous updates to the handler.
    pub fn stop_updates(&mut self) -> Result<()> {
        self.inner.stop_updates()
    }
}

/// A handler that handles location events and errors.
///
/// The handler should be registered with [`Manager::new`].
pub trait Handler: 'static + Send + Sync {
    fn handle(&self, location: Location<'_>);

    fn error(&self, error: Error);
}

/// Data about the device's current whereabouts.
///
/// Despite the name, `Location` contains more than just the location of the
/// device. See the methods for all available information.
pub struct Location<'a> {
    inner: sys::Location<'a>,
}

impl Location<'_> {
    pub fn coordinates(&self) -> Result<Coordinates> {
        self.inner.coordinates()
    }

    pub fn altitude(&self) -> Result<f64> {
        self.inner.altitude()
    }

    /// The direction in which the device is travelling, measured in degrees and
    /// relative to due north.
    pub fn bearing(&self) -> Result<f64> {
        self.inner.bearing()
    }

    /// The instantaneous speed of the device measured in meters per second.
    pub fn speed(&self) -> Result<f64> {
        self.inner.speed()
    }

    /// The time at which the location was acquired.
    ///
    /// This is not currently supported on Windows.
    pub fn time(&self) -> Result<SystemTime> {
        self.inner.time()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Copy, Clone, Debug)]
pub enum Access {
    Foreground,
    Background,
}

#[derive(Copy, Clone, Debug)]
pub enum Accuracy {
    Approximate,
    Precise,
}
