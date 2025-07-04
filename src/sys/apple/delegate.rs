use objc2::{
    define_class, msg_send, rc::Retained, AnyThread, DeclaredClass
};
use objc2_core_location::{CLError, CLLocation, CLLocationManager, CLLocationManagerDelegate};
use objc2_foundation::{NSArray, NSError, NSObject, NSObjectProtocol};

use super::Location;
use crate::{Error, Handler};

type InnerHandler = dyn Handler;

pub(super) struct Ivars {
    handler: Box<InnerHandler>,
}


define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = AnyThread] // TODO: not sure if this needs to be MainThreadOnly
    #[ivars = Ivars]
    pub(super) struct RobiusLocationDelegate;

    unsafe impl NSObjectProtocol for RobiusLocationDelegate {}

    unsafe impl CLLocationManagerDelegate for RobiusLocationDelegate {
        #[unsafe(method(locationManager:didUpdateLocations:))]
        #[allow(non_snake_case)]
        unsafe fn locationManager_didUpdateLocations(
            &self,
            _: &CLLocationManager,
            locations: &NSArray<CLLocation>,
        ) {
            for location in locations.iter() {
                self.ivars().handler.handle(
                    crate::Location {
                        inner: Location {
                            inner: &location,
                        },
                    }
                );
            }

            // for i in 0..locations.len() {
            //     self.ivars().handler.handle(crate::Location {
            //         inner: Location {
            //             // IDK why NSArray: IntoIterator doesn't work.
            //             inner: locations.get(i).unwrap(),
            //         },
            //     });
            // }
        }

        #[unsafe(method(locationManager:didFailWithError:))]
        #[allow(non_snake_case)]
        unsafe fn locationManager_didFailWithError(&self, _: &CLLocationManager, error: &NSError) {
            self.ivars().handler.error(match CLError(error.code()) {
                // kCLErrorLocationUnknown
                CLError::LocationUnknown => Error::TemporarilyUnavailable,
                // kCLErrorDenied
                CLError::Denied => Error::AuthorizationDenied,
                // kCLErrorNetwork
                CLError::Network => Error::Network,
                _ => Error::Unknown,
            })
        }
    }
);


impl RobiusLocationDelegate {
    /// Allocates a new `RobiusLocationDelegate` and initializes it with the given handler
    /// to be called upon location updates and errors.
    pub(super) fn new<T: Handler>(handler: T) -> Retained<Self> {
        let this = Self::alloc()
            .set_ivars(Ivars {
                handler: Box::new(handler),
            });
        unsafe { msg_send![super(this), init] }
    }
}



////////////////////////////////////////////////////////
////////////////////////////////////////////////////////
////////////////////////////////////////////////////////

/*
declare_class!(
    // pub(super) struct Delegate;

    // unsafe impl ClassType for Delegate {
    //     type Super = NSObject;
    //     type Mutability = mutability::InteriorMutable;
    //     const NAME: &'static str = "RobiusLocationDelegate";
    // }

    // impl DeclaredClass for Delegate {
    //     type Ivars = Ivars;
    // }

    unsafe impl Delegate {
        #[method_id(initWithHandler:)]
        fn init_with(this: Allocated<Self>, cursed: [usize; 2]) -> Option<Retained<Self>> {
            let ptr: *mut InnerHandler = unsafe { std::mem::transmute(cursed) };
            let this = this.set_ivars(Ivars {
                handler: unsafe { Box::from_raw(ptr) },
            });
            unsafe { msg_send_id![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for Delegate {}

    // #[allow(non_snake_case)]
    // unsafe impl CLLocationManagerDelegate for Delegate {
    //     #[method(locationManager:didUpdateLocations:)]
    //     fn locationManager_didUpdateLocations(
    //         &self,
    //         _: &CLLocationManager,
    //         locations: &NSArray<CLLocation>,
    //     ) {
    //         for i in 0..locations.len() {
    //             self.ivars().handler.handle(crate::Location {
    //                 inner: Location {
    //                     // IDK why NSArray: IntoIterator doesn't work.
    //                     inner: locations.get(i).unwrap(),
    //                 },
    //             });
    //         }
    //     }

    //     #[method(locationManager:didFailWithError:)]
    //     fn locationManager_didFailWithError(&self, _: &CLLocationManager, error: &NSError) {
    //         // https://github.com/theos/sdks/blob/ca52092676249546f08657d4fc0c8beb26a80510/iPhoneOS9.3.sdk/System/Library/Frameworks/CoreLocation.framework/Headers/CLError.h#L32
    //         self.ivars().handler.error(match error.code() {
    //             // kCLErrorLocationUnknown
    //             0 => Error::TemporarilyUnavailable,
    //             // kCLErrorDenied
    //             1 => Error::AuthorizationDenied,
    //             // kCLErrorNetwork
    //             2 => Error::Network,
    //             _ => Error::Unknown,
    //         })
    //     }
    // }
);

// impl Delegate {
//     pub(super) fn new<T>(handler: T) -> Retained<Self>
//     where
//         T: Handler,
//     {
//         let erased: Box<dyn Handler> = Box::new(handler);
//         let ptr: *mut InnerHandler = Box::into_raw(erased);
//         let cursed: [usize; 2] = unsafe { std::mem::transmute(ptr) };
//         unsafe { msg_send_id![Self::alloc(), initWithHandler: cursed] }
//     }
// }

*/