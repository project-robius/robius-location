mod callback;
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use jni::objects::{GlobalRef, JClass, JObject, JValueGen};
use makepad_widgets::error;

use crate::{Coordinates, Error, Handler};

type InnerHandler = Mutex<dyn Handler>;

pub struct Manager {
    // It's fine to use an `std` Mutex in an asynchronous context here, because we can only
    // encounter contention when dropping, and the guard isn't held across await points.
    handler: Arc<InnerHandler>,
}

impl Manager {
    pub fn new<T>(handler: T) -> Self
    where
        T: Handler,
    {
        Manager {
            handler: Arc::new(Mutex::new(handler)),
        }
    }

    pub fn request_authorization(&self) {
        robius_android_env::with_activity(|env, current_activity| {
            const COARSE_PERMISSION: &str = "android.permission.ACCESS_COARSE_LOCATION";
            const FINE_PERMISSION: &str = "android.permission.ACCESS_FINE_LOCATION";

            let permissions = env.new_string(FINE_PERMISSION).unwrap();
            let array = env
                .new_object_array(1, "java/lang/String", permissions)
                .unwrap();
            let request_code = 3;

            env.call_method(
                current_activity,
                "requestPermissions",
                "([Ljava/lang/String;I)V",
                &[JValueGen::Object(&array), JValueGen::Int(request_code)],
            )
            .unwrap();
        });
    }

    pub fn update_once(&self) {
        robius_android_env::with_activity(|env, context| {
            let service_name = env.new_string("location").unwrap();

            std::thread::sleep(std::time::Duration::from_secs(5));

            let manager = env
                .call_method(
                    context,
                    "getSystemService",
                    "(Ljava/lang/String;)Ljava/lang/Object;",
                    &[JValueGen::Object(&service_name)],
                )
                .unwrap()
                .l()
                .unwrap();

            let provider = env.new_string("fused").unwrap();

            let executor = env
                .call_method(
                    context,
                    "getMainExecutor",
                    "()Ljava/util/concurrent/Executor;",
                    &[],
                )
                .unwrap()
                .l()
                .unwrap();

            let class = callback::get_callback_class(env).unwrap();

            let weak_ptr: *const InnerHandler = Arc::downgrade(&self.handler).into_raw();
            // TODO: Is there a better way without the provenance API?
            let transmuted: [i64; 2] = unsafe { std::mem::transmute(weak_ptr) };
            let consumer = env
                .new_object(
                    class,
                    "(JJ)V",
                    &[
                        JValueGen::Long(transmuted[0]),
                        JValueGen::Long(transmuted[1]),
                    ],
                )
                .unwrap();

            env.call_method(
                manager,
                "getCurrentLocation",
                "(Ljava/lang/String;Landroid/os/CancellationSignal;Ljava/util/concurrent/Executor;\
                 Ljava/util/function/Consumer;)V",
                &[
                    JValueGen::Object(&provider),
                    JValueGen::Object(&JObject::null()),
                    JValueGen::Object(&executor),
                    JValueGen::Object(&consumer),
                ],
            )
            .unwrap();
        });
    }

    pub fn start_updates(&self) {
        // todo!();
    }

    pub fn stop_updates(&self) {
        // todo!();
    }
}

// TODO: Could inner be JObject<'a>?
pub struct Location<'a> {
    inner: GlobalRef,
    phantom: PhantomData<&'a ()>,
}

impl Location<'_> {
    pub fn coordinates(&self) -> Coordinates {
        robius_android_env::with_activity(|env, _| {
            let latitude = env
                .call_method(&self.inner, "getLatitude", "()D", &[])
                .unwrap()
                .d()
                .unwrap();
            let longitude = env
                .call_method(&self.inner, "getLongitude", "()D", &[])
                .unwrap()
                .d()
                .unwrap();
            Coordinates {
                latitude,
                longitude,
            }
        })
        .unwrap()
    }

    pub fn altitude(&self) -> f64 {
        todo!();
    }

    pub fn bearing(&self) -> f64 {
        todo!();
    }

    pub fn speed(&self) -> f64 {
        todo!();
    }

    pub fn time(&self) {
        todo!();
    }
}

impl From<jni::errors::Error> for Error {
    fn from(_: jni::errors::Error) -> Self {
        Error::Unknown
    }
}
