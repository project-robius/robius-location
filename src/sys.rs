cfg_if::cfg_if! {
    if #[cfg(target_os = "android")] {
        mod android;
        pub(crate) use android::*;
    } else if #[cfg(target_vendor = "apple")] {
        mod apple;
        pub(crate) use apple::*;
    } else {
        mod unsupported;
        pub(crate) use unsupported::*;
    }
}
