mod delegate;

use std::time::{Duration, SystemTime};

use delegate::Delegate;
use objc2::{rc::Id, runtime::ProtocolObject};
use objc2_core_location::{
    CLLocation, CLLocationCoordinate2D, CLLocationManager, CLLocationManagerDelegate,
};

use crate::{Access, Accuracy, Coordinates, Handler, Result};

pub(crate) struct Manager {
    inner: Id<CLLocationManager>,
    // We don't want to drop the handler until the manager is dropped.
    _delegate: Id<ProtocolObject<dyn CLLocationManagerDelegate>>,
}

impl Manager {
    pub(crate) fn new<T>(handler: T) -> Result<Self>
    where
        T: Handler,
    {
        let inner = unsafe { CLLocationManager::new() };
        let delegate = ProtocolObject::from_id(Delegate::new(handler));
        unsafe { inner.setDelegate(Some(&delegate)) };
        Ok(Self {
            inner,
            _delegate: delegate,
        })
    }

    pub(crate) fn request_authorization(&self, access: Access, _: Accuracy) -> Result<()> {
        match access {
            Access::Foreground => unsafe { self.inner.requestWhenInUseAuthorization() },
            Access::Background => unsafe { self.inner.requestAlwaysAuthorization() },
        }
        Ok(())
    }

    pub(crate) fn update_once(&self) -> Result<()> {
        unsafe { self.inner.requestLocation() };
        Ok(())
    }

    pub(crate) fn start_updates(&self) -> Result<()> {
        unsafe { self.inner.startUpdatingLocation() }
        Ok(())
    }

    pub(crate) fn stop_updates(&self) -> Result<()> {
        unsafe { self.inner.stopUpdatingLocation() }
        Ok(())
    }
}

pub(crate) struct Location<'a> {
    inner: &'a CLLocation,
}

impl Location<'_> {
    pub(crate) fn coordinates(&self) -> Result<Coordinates> {
        let CLLocationCoordinate2D {
            latitude,
            longitude,
        } = unsafe { self.inner.coordinate() };

        Ok(Coordinates {
            latitude,
            longitude,
        })
    }

    pub(crate) fn altitude(&self) -> Result<f64> {
        Ok(unsafe { self.inner.altitude() })
    }

    pub(crate) fn bearing(&self) -> Result<f64> {
        Ok(unsafe { self.inner.course() })
    }

    pub(crate) fn speed(&self) -> Result<f64> {
        Ok(unsafe { self.inner.speed() })
    }

    pub(crate) fn time(&self) -> Result<SystemTime> {
        let secs = unsafe { self.inner.timestamp().timeIntervalSince1970() };
        Ok(SystemTime::UNIX_EPOCH + Duration::from_secs_f64(secs))
    }
}
