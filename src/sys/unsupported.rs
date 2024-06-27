use std::marker::PhantomData;

use crate::{Coordinates, Handler, Result};

pub(crate) struct Manager;

impl Manager {
    pub fn new<T>(_handler: T) -> Result<Self>
    where
        T: Handler,
    {
        Ok(Self)
    }

    pub fn request_authorization(&self) -> Result<()> {
        Ok(())
    }

    pub fn update_once(&self) -> Result<()> {
        Ok(())
    }

    pub fn start_updates(&self) -> Result<()> {
        Ok(())
    }

    pub fn stop_updates(&self) -> Result<()> {
        Ok(())
    }
}

pub struct Location<'a> {
    _phantom_data: PhantomData<&'a ()>,
}

impl Location<'_> {
    pub fn coordinates(&self) -> Result<Coordinates> {
        unimplemented!();
    }

    pub fn altitude(&self) -> Result<f64> {
        unimplemented!();
    }

    pub fn bearing(&self) -> Result<f64> {
        unimplemented!();
    }

    pub fn speed(&self) -> Result<f64> {
        unimplemented!();
    }

    pub fn time(&self) {
        unimplemented!();
    }
}
