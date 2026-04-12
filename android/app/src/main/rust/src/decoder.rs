use ffmpeg_next as ffmpeg;
use anyhow::Result;

pub struct Decoder {
    format_ctx: ffmpeg::format::context::Input,
    decoder: ffmpeg::decoder::Audio,
    stream_index: usize,
    resampler: ffmpeg::software::resampling::Context,
    duration_ms: i64,
    is_finished: bool,
    // Add fields for speed control if using ffmpeg filters
}

impl Decoder {
    pub fn new(url: &str, sample_rate: u32, channels: usize) -> Result<Self> {
        ffmpeg::init()?;

        let format_ctx = ffmpeg::format::input(&url)?;
        let stream = format_ctx
            .streams()
            .best(ffmpeg::media::Type::Audio)
            .ok_or_else(|| anyhow::anyhow!("No audio stream found"))?;
        let stream_index = stream.index();

        let context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
        let decoder = context.decoder().audio()?;

        let resampler = ffmpeg::software::resampling::Context::get(
            decoder.format(),
            decoder.channel_layout(),
            decoder.rate(),
            ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed),
            if channels == 1 { ffmpeg::util::channel_layout::ChannelLayout::MONO } else { ffmpeg::util::channel_layout::ChannelLayout::STEREO },
            sample_rate,
        )?;

        let duration_ms = if let Some(d) = format_ctx.duration() {
            (d as f64 / 1000.0) as i64 // context.duration() is in AV_TIME_BASE (microseconds)
        } else {
            0
        };

        Ok(Self {
            format_ctx,
            decoder,
            stream_index,
            resampler,
            duration_ms,
            is_finished: false,
        })
    }

    pub fn next_samples(&mut self, data: &mut [f32]) -> Result<usize> {
        // Implement frame decoding and resampling
        // This is a simplified placeholder
        Ok(0)
    }

    pub fn seek(&mut self, pos_ms: i64) -> Result<()> {
        let timestamp = (pos_ms as f64 / 1000.0 * ffmpeg::ffi::AV_TIME_BASE as f64) as i64;
        self.format_ctx.seek(timestamp, ..timestamp)?;
        self.decoder.flush();
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
        // Implement via ffmpeg filters
    }
}
