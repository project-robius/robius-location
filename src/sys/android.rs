mod callback;

use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use jni::{
    objects::{GlobalRef, JObject, JValueGen},
    JNIEnv,
};

use crate::{Coordinates, Error, Handler, Result};

type InnerHandler = Mutex<dyn Handler>;

pub struct Manager {
    callback: GlobalRef,
    // It's fine to use an `std` Mutex in an asynchronous context here, because we can only
    // encounter contention when dropping, and the guard isn't held across await points.
    //
    // We store this so that it is not dropped, and the callback can call the handler.
    _handler: Arc<InnerHandler>,
}

impl Manager {
    pub fn new<T>(handler: T) -> Self
    where
        T: Handler,
    {
        let handler: Arc<InnerHandler> = Arc::new(Mutex::new(handler));
        Manager {
            callback: robius_android_env::with_activity(|env, _| {
                let callback = construct_callback(env, &handler);
                env.new_global_ref(callback).unwrap()
            })
            .unwrap(),
            _handler: handler,
        }
    }

    pub fn request_authorization(&self) -> Result<()> {
        robius_android_env::with_activity(|env, current_activity| {
            // const COARSE_PERMISSION: &str = "android.permission.ACCESS_COARSE_LOCATION";
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

        // TODO
        Ok(())
    }

    pub fn update_once(&self) {
        robius_android_env::with_activity(|env, context| {
            let manager = get_location_manager(env, context);
            let provider = env.new_string("fused").unwrap();
            let executor = get_executor(env, context);

            env.call_method(
                manager,
                "getCurrentLocation",
                "(Ljava/lang/String;Landroid/os/CancellationSignal;Ljava/util/concurrent/Executor;\
                 Ljava/util/function/Consumer;)V",
                &[
                    JValueGen::Object(&provider),
                    JValueGen::Object(&JObject::null()),
                    JValueGen::Object(&executor),
                    JValueGen::Object(&self.callback),
                ],
            )
            .unwrap();
        });
    }

    pub fn start_updates(&self) {
        // TODO: What happens if user calls start_updates multiple times?

        // TODO: NoClassDefFoundError for android/location/LocationListener$-CC

        robius_android_env::with_activity(|env, context| {
            let result = env.find_class("android/location/LocationListener");
            makepad_widgets::log!("1: {result:?}");
            let manager = get_location_manager(env, context);
            let provider = env.new_string("fused").unwrap();
            let request = construct_location_request(env);
            let executor = get_executor(env, context);

            let result = env.call_method(
                manager,
                "requestLocationUpdates",
                "(Ljava/lang/String;Landroid/location/LocationRequest;Ljava/util/concurrent/\
                 Executor;Landroid/location/LocationListener;)V",
                &[
                    JValueGen::Object(&provider),
                    JValueGen::Object(&request),
                    JValueGen::Object(&executor),
                    JValueGen::Object(&self.callback),
                ],
            );
            makepad_widgets::log!("2: {result:?}");
        });
    }

    pub fn stop_updates(&self) {
        // TODO: Request flush?

        // TODO: What happens if user calls stop_updates prior to calling start_updates

        robius_android_env::with_activity(|env, context| {
            let manager = get_location_manager(env, context);
            env.call_method(
                manager,
                "removeUpdates",
                "(Landroid/location/LocationListener;)V",
                &[JValueGen::Object(&self.callback)],
            )
            .unwrap();
        });
    }
}

fn get_location_manager<'a>(env: &mut JNIEnv<'a>, context: &JObject<'_>) -> JObject<'a> {
    let service_name = env.new_string("location").unwrap();

    env.call_method(
        context,
        "getSystemService",
        "(Ljava/lang/String;)Ljava/lang/Object;",
        &[JValueGen::Object(&service_name)],
    )
    .unwrap()
    .l()
    .unwrap()
}

fn get_executor<'a>(env: &mut JNIEnv<'a>, context: &JObject<'_>) -> JObject<'a> {
    env.call_method(
        context,
        "getMainExecutor",
        "()Ljava/util/concurrent/Executor;",
        &[],
    )
    .unwrap()
    .l()
    .unwrap()
}

fn construct_callback<'a>(env: &mut JNIEnv<'a>, handler: &Arc<InnerHandler>) -> JObject<'a> {
    let callback_class = callback::get_callback_class(env).unwrap();

    let weak_ptr: *const InnerHandler = Arc::downgrade(handler).into_raw();
    // TODO: Is there a better way without the provenance API?
    let transmuted: [i64; 2] = unsafe { std::mem::transmute(weak_ptr) };
    env.new_object(
        callback_class,
        "(JJ)V",
        &[
            JValueGen::Long(transmuted[0]),
            JValueGen::Long(transmuted[1]),
        ],
    )
    .unwrap()
}

fn construct_location_request<'a>(env: &mut JNIEnv<'a>) -> JObject<'a> {
    let builder = env
        .new_object(
            "android/location/LocationRequest$Builder",
            "(J)V",
            // TODO: Don't hardcode
            &[JValueGen::Long(100)],
        )
        .unwrap();

    env.call_method(
        builder,
        "build",
        "()Landroid/location/LocationRequest;",
        &[],
    )
    .unwrap()
    .l()
    .unwrap()
}

// TODO: Could inner be JObject<'a>?
pub struct Location<'a> {
    inner: GlobalRef,
    phantom: PhantomData<&'a ()>,
}

impl Location<'_> {
    pub fn coordinates(&self) -> Result<Coordinates> {
        robius_android_env::with_activity(|env, _| {
            let latitude = env
                .call_method(&self.inner, "getLatitude", "()D", &[])?
                .d()?;
            let longitude = env
                .call_method(&self.inner, "getLongitude", "()D", &[])?
                .d()?;
            Ok(Coordinates {
                latitude,
                longitude,
            })
        })
        .ok_or(Error::AndroidEnvironment)
        // Poor man's `flatten`
        .and_then(|x| x)
    }

    pub fn altitude(&self) -> Result<f64> {
        robius_android_env::with_activity(|env, _| {
            env.call_method(&self.inner, "getAltitude", "()D", &[])?
                .d()
                .map_err(|e| e.into())
        })
        .ok_or(Error::AndroidEnvironment)
        .and_then(|x| x)
    }

    pub fn bearing(&self) -> Result<f64> {
        robius_android_env::with_activity(|env, _| {
            match env.call_method(&self.inner, "getBearing", "()F", &[])?.f() {
                Ok(bearing) => Ok(bearing as f64),
                Err(e) => Err(e.into()),
            }
        })
        .ok_or(Error::AndroidEnvironment)
        .and_then(|x| x)
    }

    pub fn speed(&self) -> Result<f64> {
        robius_android_env::with_activity(|env, _| {
            match env.call_method(&self.inner, "getSpeed", "()F", &[])?.f() {
                Ok(speed) => Ok(speed as f64),
                Err(e) => Err(e.into()),
            }
        })
        .ok_or(Error::AndroidEnvironment)
        .and_then(|x| x)
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
