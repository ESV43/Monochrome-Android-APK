use symphonia::core::formats::{FormatReader, FormatOptions, SeekMode, SeekTo};
use symphonia::core::codecs::{Decoder as SymphoniaDecoder, DecoderOptions};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::core::audio::SampleBuffer;
use anyhow::{Result, anyhow};
use std::fs::File;

pub struct Decoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn SymphoniaDecoder>,
    sample_buffer: Option<SampleBuffer<f32>>,
    duration_ms: i64,
    is_finished: bool,
    current_sample_offset: usize,
}

impl Decoder {
    pub fn new(url: &str, sample_rate: u32, _channels: usize) -> Result<Self> {
        let src = if url.starts_with("http") {
             return Err(anyhow!("HTTP streaming not yet implemented in Symphonia decoder (use local file)"));
        } else {
            File::open(url)?
        };

        let mss = MediaSourceStream::new(Box::new(src), Default::default());
        let hint = Hint::new();
        let format_opts = FormatOptions::default();
        let metadata_opts = Default::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)?;

        let format = probed.format;
        let track = format.tracks()
            .iter()
            .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            .ok_or_else(|| anyhow!("No supported audio track found"))?;

        let decoder_opts = DecoderOptions::default();
        let decoder = symphonia::default::get_codecs().make(&track.codec_params, &decoder_opts)?;

        let duration_ms = track.codec_params.n_frames
            .map(|n| (n as f64 / track.codec_params.sample_rate.unwrap_or(44100) as f64 * 1000.0) as i64)
            .unwrap_or(0);

        Ok(Self {
            format,
            decoder,
            sample_buffer: None,
            duration_ms,
            is_finished: false,
            current_sample_offset: 0,
        })
    }

    pub fn next_samples(&mut self, data: &mut [f32]) -> Result<usize> {
        if self.is_finished { return Ok(0); }

        let mut samples_written = 0;

        while samples_written < data.len() {
            // Use buffered samples if available
            if let Some(ref buffer) = self.sample_buffer {
                let available = buffer.samples().len() - self.current_sample_offset;
                let to_copy = std::cmp::min(available, data.len() - samples_written);
                
                data[samples_written..samples_written + to_copy].copy_from_slice(
                    &buffer.samples()[self.current_sample_offset..self.current_sample_offset + to_copy]
                );
                
                samples_written += to_copy;
                self.current_sample_offset += to_copy;

                if self.current_sample_offset >= buffer.samples().len() {
                    self.sample_buffer = None;
                    self.current_sample_offset = 0;
                }
                continue;
            }

            // Decode next packet
            match self.format.next_packet() {
                Ok(packet) => {
                    let decoded = self.decoder.decode(&packet)?;
                    let spec = *decoded.spec();
                    let duration = decoded.capacity() as u64;

                    let mut buffer = SampleBuffer::<f32>::new(duration, spec);
                    buffer.copy_interleaved_ref(decoded);
                    self.sample_buffer = Some(buffer);
                    self.current_sample_offset = 0;
                }
                Err(symphonia::core::errors::Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    self.is_finished = true;
                    break;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(samples_written)
    }

    pub fn seek(&mut self, pos_ms: i64) -> Result<()> {
        self.format.seek(SeekMode::Coarse, SeekTo::Time {
            time: symphonia::core::units::Time::from(pos_ms as u64 / 1000),
            track_id: None,
        })?;
        self.decoder.reset();
        self.sample_buffer = None;
        self.current_sample_offset = 0;
        self.is_finished = false;
        Ok(())
    }

    pub fn duration_ms(&self) -> i64 {
        self.duration_ms
    }

    pub fn is_finished(&self) -> bool {
        self.is_finished
    }

    pub fn set_speed(&mut self, _speed: f32) {
        // Speed control can be implemented via a DSP node later
    }
}
