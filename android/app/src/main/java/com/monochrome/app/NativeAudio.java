package com.monochrome.app;

/**
 * JNI wrapper for the native Rust audio engine.
 */
public class NativeAudio {
    static {
        System.loadLibrary("monochrome_native_audio");
    }

    public static native void init();
    public static native void play(String url);
    public static native void pause();
    public static native void resume();
    public static native void stop();
    public static native void seek(long posMs);
    public static native void setVolume(float volume);
    public static native void setEqGains(float[] gains);
    public static native void setSpeed(float speed);
    public static native long getPosition();
    public static native long getDuration();
}
