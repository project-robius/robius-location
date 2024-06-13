use std::{marker::PhantomData, mem::transmute, sync::OnceLock};

use jni::{
    objects::{GlobalRef, JClass, JObject, JValueGen},
    sys::jlong,
    JNIEnv, NativeMethod,
};
use tokio::sync::oneshot;

use crate::{Error, Handler, Result};

const AUTHENTICATION_CALLBACK_BYTECODE: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/classes.dex"));

type ChannelData = Result<()>;

pub(super) type Receiver = oneshot::Receiver<ChannelData>;
pub(super) type Sender = oneshot::Sender<ChannelData>;

pub(super) fn channel() -> (Sender, Receiver) {
    oneshot::channel()
}

// NOTE: This must be kept in sync with the signature of `rust_callback`.
const RUST_CALLBACK_SIGNATURE: &str = "(JJJLandroid/location/Location;)V";

// NOTE: The signature of this function must be kept in sync with
// `RUST_CALLBACK_SIGNATURE`.
unsafe extern "C" fn rust_callback<'a>(
    env: JNIEnv<'a>,
    _: JObject<'a>,
    handler_ptr: jlong,
    handler_fn_ptr: jlong,
    handler_err_fn_ptr: jlong,
    location: JObject<'a>,
) {
    // TODO: 32-bit? What's that?

    // FIXME: What if dropped on main thread?

    let handler: &mut () = unsafe { transmute(handler_ptr) };
    // TODO: Note that this MUST be kept in sync with the handler function signature
    // :) Would be nice if we could somehow statically check this at compile
    // time.
    let handler_fn: for<'b> fn(&mut (), crate::Location<'b>) = unsafe { transmute(handler_fn_ptr) };
    // let handler_err_fn = unsafe { transmute(handler_err_fn_ptr) };

    handler_fn(
        handler,
        crate::Location {
            inner: super::Location {
                inner: env.new_global_ref(location).unwrap(),
                phantom: PhantomData,
            },
        },
    )
}

static CALLBACK_CLASS: OnceLock<GlobalRef> = OnceLock::new();

pub(super) fn get_callback_class(env: &mut JNIEnv<'_>) -> Result<&'static GlobalRef> {
    // TODO: This can be optimised when the `once_cell_try` feature is stabilised.

    if let Some(class) = CALLBACK_CLASS.get() {
        return Ok(class);
    }
    let callback_class = load_callback_class(env)?;
    register_rust_callback(env, &callback_class)?;
    let global = env.new_global_ref(callback_class)?;

    Ok(CALLBACK_CLASS.get_or_init(|| global))
}

fn register_rust_callback<'a>(env: &mut JNIEnv<'a>, callback_class: &JClass<'a>) -> Result<()> {
    env.register_native_methods(
        callback_class,
        &[NativeMethod {
            name: "rustCallback".into(),
            sig: RUST_CALLBACK_SIGNATURE.into(),
            fn_ptr: rust_callback as *mut _,
        }],
    )
    .map_err(|e| e.into())
}

fn load_callback_class<'a>(env: &mut JNIEnv<'a>) -> Result<JClass<'a>> {
    const LOADER_CLASS: &str = "dalvik/system/InMemoryDexClassLoader";

    let byte_buffer = unsafe {
        env.new_direct_byte_buffer(
            AUTHENTICATION_CALLBACK_BYTECODE.as_ptr() as *mut u8,
            AUTHENTICATION_CALLBACK_BYTECODE.len(),
        )
    }?;

    let dex_class_loader = env.new_object(
        LOADER_CLASS,
        "(Ljava/nio/ByteBuffer;Ljava/lang/ClassLoader;)V",
        &[
            JValueGen::Object(&JObject::from(byte_buffer)),
            JValueGen::Object(&JObject::null()),
        ],
    )?;

    Ok(env
        .call_method(
            &dex_class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValueGen::Object(&JObject::from(
                env.new_string("robius/location/LocationCallback").unwrap(),
            ))],
        )?
        .l()?
        .into())
}
