use jni::objects::{JClass, JFloatArray, JString};
use jni::sys::{jfloat, jlong};
use jni::JNIEnv;
use parking_lot::Mutex;
use std::sync::Arc;
use once_cell::sync::Lazy;
use crate::audio_engine::Player;

static PLAYER: Lazy<Arc<Mutex<Option<Player>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_monochrome_app_NativeAudio_init(
    _env: JNIEnv,
    _class: JClass,
) {
    let mut player = PLAYER.lock();
    if player.is_none() {
        match Player::new() {
            Ok(p) => *player = Some(p),
            Err(e) => log::error!("Failed to initialize player: {:?}", e),
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_monochrome_app_NativeAudio_play(
    mut env: JNIEnv,
    _class: JClass,
    url: JString,
) {
    let url_str: String = match env.get_string(&url) {
        Ok(s) => s.into(),
        Err(_) => return,
    };

    if let Some(player) = PLAYER.lock().as_ref() {
        if let Err(e) = player.load(&url_str) {
            log::error!("Failed to load URL {}: {:?}", url_str, e);
        } else {
            player.play();
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_monochrome_app_NativeAudio_pause(
    _env: JNIEnv,
    _class: JClass,
) {
    if let Some(player) = PLAYER.lock().as_ref() {
        player.pause();
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_monochrome_app_NativeAudio_resume(
    _env: JNIEnv,
    _class: JClass,
) {
    if let Some(player) = PLAYER.lock().as_ref() {
        player.play();
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_monochrome_app_NativeAudio_stop(
    _env: JNIEnv,
    _class: JClass,
) {
    if let Some(player) = PLAYER.lock().as_ref() {
        player.stop();
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_monochrome_app_NativeAudio_seek(
    _env: JNIEnv,
    _class: JClass,
    pos_ms: jlong,
) {
    if let Some(player) = PLAYER.lock().as_ref() {
        player.seek(pos_ms);
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_monochrome_app_NativeAudio_setVolume(
    _env: JNIEnv,
    _class: JClass,
    volume: jfloat,
) {
    if let Some(player) = PLAYER.lock().as_ref() {
        player.set_volume(volume);
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_monochrome_app_NativeAudio_setEqGains(
    mut env: JNIEnv,
    _class: JClass,
    gains: JFloatArray,
) {
    let mut buffer = vec![0.0f32; 0];
    if let Ok(len) = env.get_array_length(&gains) {
        buffer.resize(len as usize, 0.0);
        if let Err(e) = env.get_float_array_region(&gains, 0, &mut buffer) {
            log::error!("Failed to get float array region: {:?}", e);
            return;
        }
    }

    if let Some(player) = PLAYER.lock().as_ref() {
        player.set_eq_gains(&buffer);
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_monochrome_app_NativeAudio_setSpeed(
    _env: JNIEnv,
    _class: JClass,
    speed: jfloat,
) {
    if let Some(player) = PLAYER.lock().as_ref() {
        player.set_speed(speed);
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_monochrome_app_NativeAudio_getPosition(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    if let Some(player) = PLAYER.lock().as_ref() {
        player.get_position()
    } else {
        0
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_monochrome_app_NativeAudio_getDuration(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    if let Some(player) = PLAYER.lock().as_ref() {
        player.get_duration()
    } else {
        0
    }
}
