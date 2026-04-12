pub mod audio_engine;
pub mod decoder;
pub mod dsp;
pub mod jni_bridge;

use android_logger::Config;
use log::LevelFilter;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn JNI_OnLoad(_vm: jni::JavaVM, _reserved: std::ffi::c_void) -> jni::sys::jint {
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Debug)
            .with_tag("MonochromeNativeAudio"),
    );
    jni::sys::JNI_VERSION_1_6
}
