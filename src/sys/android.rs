mod callback;
use std::marker::PhantomData;

use jni::objects::{GlobalRef, JClass, JObject, JValueGen};
use makepad_widgets::error;

use crate::{Coordinates, Error, Handler};

pub struct Manager {
    handler: Box<dyn Handler>,
    // TODO: Ideally we could recreate these from the above Box. It should technically be possible
    // as they are stored in the vtable, but I can't seem to get it to work. We may need to use a
    // custom vtable rather than a `Box<dyn Handler>`.
    fn_ptr: i64,
    err_fn_ptr: i64,
}

impl Manager {
    pub fn new<T>(handler: T) -> Self
    where
        T: Handler,
    {
        Manager {
            handler: Box::new(handler),
            fn_ptr: T::handle as i64,
            err_fn_ptr: T::error as i64,
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

            // 🙃
            let fat_ptr: [usize; 2] = unsafe { core::mem::transmute(&*self.handler) };
            // TODO: We are assuming the first component of the fat pointer points to the
            // struct, and the second points to the vtable.
            let thin_ptr = fat_ptr[0] as *const () as i64;
            let consumer = env
                .new_object(
                    class,
                    "(JJJ)V",
                    &[
                        JValueGen::Long(thin_ptr),
                        JValueGen::Long(self.fn_ptr),
                        JValueGen::Long(self.err_fn_ptr),
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
