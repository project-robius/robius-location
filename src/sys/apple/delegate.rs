use icrate::{
    objc2::{
        declare_class, msg_send_id, mutability,
        rc::{Allocated, Id},
        ClassType, DeclaredClass,
    },
    CoreLocation::{CLLocation, CLLocationManager, CLLocationManagerDelegate},
    Foundation::{NSArray, NSError, NSObject, NSObjectProtocol},
};

use super::Location;
use crate::Handler;

pub(super) struct Ivars {
    handler: Box<dyn Handler>,
}

declare_class!(
    pub(super) struct Delegate;

    unsafe impl ClassType for Delegate {
        type Super = NSObject;
        type Mutability = mutability::InteriorMutable;
        const NAME: &'static str = "RobiusLocationDelegate";
    }

    impl DeclaredClass for Delegate {
        type Ivars = Ivars;
    }

    unsafe impl Delegate {
        #[method_id(initWithHandler:)]
        fn init_with(this: Allocated<Self>, cursed: [usize; 2]) -> Option<Id<Self>> {
            let this = this.set_ivars(Ivars {
                // FIXME TODO NOTE XXX: :P
                // Provenance be damned.
                handler: unsafe { core::mem::transmute(cursed) },
            });
            unsafe { msg_send_id![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for Delegate {}

    #[allow(non_snake_case)]
    unsafe impl CLLocationManagerDelegate for Delegate {
        #[method(locationManager:didUpdateLocations:)]
        fn locationManager_didUpdateLocations(
            &self,
            _: &CLLocationManager,
            locations: &NSArray<CLLocation>,
        ) {
            let most_recent = locations.last();
            self.ivars().handler.something(crate::Location {
                inner: Location {
                    // TODO: Is there guaranteed to be at least one location?
                    inner: most_recent.unwrap(),
                },
            });
        }

        #[method(locationManager:didFailWithError:)]
        fn locationManager_didFailWithError(&self, _: &CLLocationManager, _: &NSError) {
            // TODO: Match on error
            self.ivars().handler.error(crate::Error::Unknown)
        }
    }
);

impl Delegate {
    pub(super) fn new<T>(handler: T) -> Id<Self>
    where
        T: Handler,
    {
        let erased: Box<dyn Handler> = Box::new(handler);
        let ptr = Box::into_raw(erased);
        let cursed: [usize; 2] = unsafe { std::mem::transmute(ptr) };
        unsafe { msg_send_id![Self::alloc(), initWithHandler: cursed] }
    }
}
