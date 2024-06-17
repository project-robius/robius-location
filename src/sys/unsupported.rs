use std::marker::PhantomData;

use crate::{Coordinates, Handler, Result};

pub(crate) struct Manager;

impl Manager {
    pub fn new<T>(_handler: T) -> Self
    where
        T: Handler,
    {
        Self
    }

    pub fn request_authorization(&self) -> Result<()> {
        Ok(())
    }

    pub fn update_once(&self) {}

    pub fn start_updates(&self) {}

    pub fn stop_updates(&self) {}
}

pub struct Location<'a> {
    _phantom_data: PhantomData<&'a ()>,
}

impl Location<'_> {
    pub fn coordinates(&self) -> Coordinates {
        unimplemented!();
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
