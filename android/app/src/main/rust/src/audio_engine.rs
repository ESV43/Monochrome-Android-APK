use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;
use parking_lot::Mutex;
use crate::decoder::Decoder;
use crate::dsp::DspPipeline;

pub struct Player {
    #[allow(dead_code)]
    host: cpal::Host,
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
    stream: Mutex<Option<cpal::Stream>>,
    state: Arc<Mutex<PlayerState>>,
}

struct PlayerState {
    decoder: Option<Decoder>,
    dsp: DspPipeline,
    volume: f32,
    speed: f32,
    playing: bool,
    position_ms: i64,
}

impl Player {
    pub fn new() -> Result<Self, anyhow::Error> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No output device found"))?;
        let config = device.default_output_config()?;

        let state = Arc::new(Mutex::new(PlayerState {
            decoder: None,
            dsp: DspPipeline::new(config.channels() as usize, config.sample_rate().0),
            volume: 1.0,
            speed: 1.0,
            playing: false,
            position_ms: 0,
        }));

        Ok(Self {
            host,
            device,
            config,
            stream: Mutex::new(None),
            state,
        })
    }

    pub fn load(&self, url: &str) -> Result<(), anyhow::Error> {
        let mut state = self.state.lock();
        let decoder = Decoder::new(url, self.config.sample_rate().0, self.config.channels() as usize)?;
        state.decoder = Some(decoder);
        state.position_ms = 0;
        Ok(())
    }

    pub fn play(&self) {
        let mut state = self.state.lock();
        state.playing = true;
        let mut stream_lock = self.stream.lock();
        if stream_lock.is_none() {
            // Start stream if not already started
            drop(state);
            if let Some(s) = self.start_stream() {
                *stream_lock = Some(s);
            }
        }
    }

    pub fn pause(&self) {
        let mut state = self.state.lock();
        state.playing = false;
    }

    pub fn stop(&self) {
        let mut state = self.state.lock();
        state.playing = false;
        state.decoder = None;
        state.position_ms = 0;
    }

    pub fn seek(&self, pos_ms: i64) {
        let mut state = self.state.lock();
        if let Some(ref mut decoder) = state.decoder {
            if let Err(e) = decoder.seek(pos_ms) {
                log::error!("Seek failed: {:?}", e);
            } else {
                state.position_ms = pos_ms;
            }
        }
    }

    pub fn set_volume(&self, volume: f32) {
        let mut state = self.state.lock();
        state.volume = volume;
    }

    pub fn set_eq_gains(&self, gains: &[f32]) {
        let mut state = self.state.lock();
        state.dsp.set_eq_gains(gains);
    }

    pub fn set_speed(&self, speed: f32) {
        let mut state = self.state.lock();
        state.speed = speed;
        if let Some(ref mut decoder) = state.decoder {
            decoder.set_speed(speed);
        }
    }

    pub fn get_position(&self) -> i64 {
        let state = self.state.lock();
        state.position_ms
    }

    pub fn get_duration(&self) -> i64 {
        let state = self.state.lock();
        state.decoder.as_ref().map(|d| d.duration_ms()).unwrap_or(0)
    }

    fn start_stream(&self) -> Option<cpal::Stream> {
        let state_clone = self.state.clone();
        let channels = self.config.channels() as usize;

        let err_callback = |err| log::error!("An error occurred on the audio stream: {}", err);

        let stream = match self.config.sample_format() {
            cpal::SampleFormat::F32 => self.device.build_output_stream(
                &self.config.clone().into(),
                move |data: &mut [f32], _| self.write_audio_raw(data, &state_clone, channels),
                err_callback,
                None,
            ),
            _ => {
                log::error!("Unsupported sample format");
                return None;
            }
        };

        match stream {
            Ok(s) => {
                if let Err(e) = s.play() {
                    log::error!("Failed to play stream: {}", e);
                    None
                } else {
                    Some(s)
                }
            }
            Err(e) => {
                log::error!("Failed to build output stream: {}", e);
                None
            }
        }
    }

    fn write_audio_raw(&self, data: &mut [f32], state_arc: &Arc<Mutex<PlayerState>>, channels: usize) {
        let mut state = state_arc.lock();
        if !state.playing {
            data.fill(0.0);
            return;
        }

        if let Some(ref mut decoder) = state.decoder {
            let mut samples_read = 0;
            while samples_read < data.len() {
                match decoder.next_samples(&mut data[samples_read..]) {
                    Ok(n) if n > 0 => {
                        samples_read += n;
                    }
                    _ => break,
                }
            }

            // Apply DSP
            state.dsp.process(&mut data[..samples_read]);

            // Apply Volume
            let vol = state.volume;
            for sample in &mut data[..samples_read] {
                *sample *= vol;
            }

            // Update position (approximate)
            let samples_per_ms = (self.config.sample_rate().0 as f32 / 1000.0) * (channels as f32);
            let read_f = samples_read as f32;
            state.position_ms += (read_f / samples_per_ms) as i64;

            if samples_read < data.len() {
                data[samples_read..].fill(0.0);
            }

            if decoder.is_finished() {
                state.playing = false;
            }
        } else {
            data.fill(0.0);
        }
    }
}

// cpal::Stream is not Send on some Android backends, but we manage it safely
unsafe impl Send for Player {}
unsafe impl Sync for Player {}

