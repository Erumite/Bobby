use id3::TagLike;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};


#[derive(Debug, Clone)]
pub struct TrackAudioInfo {
    pub format: String,
    pub bitrate_kbps: u32,
    pub channels: u16,
}

impl TrackAudioInfo {
    pub fn display_string(&self) -> String {
        let ch_str = match self.channels {
            1 => "Mono",
            2 => "Stereo",
            n => return format!("{} {} Kbps {}Ch", self.format, self.bitrate_kbps, n),
        };
        format!("{} {} Kbps {}", self.format, self.bitrate_kbps, ch_str)
    }
}

pub struct AudioPlayer {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    sink: Option<Sink>,
    volume: f32,
    muted: bool,
    start_time: Option<Instant>,
    seek_offset: Duration,
    duration: Option<Duration>,
    playing_path: Option<String>,
    track_info: Option<TrackAudioInfo>,
    is_paused: Arc<AtomicBool>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => {
                eprintln!("Failed to initialize audio output stream: {}", e);
                (None, None)
            }
        };

        Self {
            _stream,
            stream_handle,
            sink: None,
            volume: 1.0,
            muted: false,
            start_time: None,
            seek_offset: Duration::ZERO,
            duration: None,
            playing_path: None,
            track_info: None,
            is_paused: Arc::new(AtomicBool::new(false)),
        }
    }

    fn effective_volume(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.volume
        }
    }

    pub fn play(&mut self, path: &Path) -> Result<(), String> {
        self.stop();

        let handle = match &self.stream_handle {
            Some(h) => h,
            None => return Err("Audio device not available".to_string()),
        };

        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let file_size = file.metadata().ok().map(|m| m.len()).unwrap_or(0);
        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader).map_err(|e| format!("Failed to decode audio: {}", e))?;
        let channels = decoder.channels();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("AUD").to_uppercase();
        let total_dur = decoder.total_duration().or_else(|| {
            if ext == "MP3" {
                mp3_duration::from_path(path).ok()
                    .or_else(|| {
                        id3::Tag::read_from_path(path).ok()
                            .and_then(|t| t.duration())
                            .map(|ms| Duration::from_millis(ms as u64))
                    })
                    .or_else(|| get_mp3_duration_robust(path))
            } else {
                None
            }
        });


        let bitrate_kbps = if let Some(dur) = total_dur {
            let secs = dur.as_secs_f64();
            if secs > 0.0 && file_size > 0 {
                ((file_size as f64 * 8.0) / secs / 1000.0).round() as u32
            } else {
                0
            }
        } else {
            0
        };

        let sink = Sink::try_new(handle).map_err(|e| format!("Failed to create sink: {}", e))?;
        sink.set_volume(self.effective_volume());
        sink.append(decoder);
        sink.play();

        self.sink = Some(sink);
        self.duration = total_dur;
        self.seek_offset = Duration::ZERO;
        self.start_time = Some(Instant::now());
        self.playing_path = Some(path.to_string_lossy().to_string());
        self.track_info = Some(TrackAudioInfo {
            format: ext,
            bitrate_kbps,
            channels,
        });
        self.is_paused.store(false, Ordering::SeqCst);

        Ok(())
    }

    pub fn toggle_pause(&mut self) {
        if let Some(sink) = &self.sink {
            if sink.is_paused() {
                sink.play();
                self.start_time = Some(Instant::now());
                self.is_paused.store(false, Ordering::SeqCst);
            } else {
                sink.pause();
                if let Some(start) = self.start_time {
                    self.seek_offset += start.elapsed();
                }
                self.start_time = None;
                self.is_paused.store(true, Ordering::SeqCst);
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.duration = None;
        self.seek_offset = Duration::ZERO;
        self.start_time = None;
        self.playing_path = None;
        self.track_info = None;
        self.is_paused.store(false, Ordering::SeqCst);
    }

    pub fn track_info(&self) -> Option<&TrackAudioInfo> {
        self.track_info.as_ref()
    }

    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    pub fn get_pos(&self) -> Duration {
        if let Some(start) = self.start_time {
            if self.is_playing() {
                self.seek_offset + start.elapsed()
            } else {
                self.seek_offset
            }
        } else {
            self.seek_offset
        }
    }

    pub fn seek_to(&mut self, pos: Duration) {
        if let Some(sink) = &self.sink {
            if sink.try_seek(pos).is_ok() {
                self.seek_offset = pos;
                self.start_time = Some(Instant::now());
                return;
            }
        }

        if let Some(ref path_str) = self.playing_path.clone() {
            let path = Path::new(path_str);
            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                if let Ok(decoder) = Decoder::new(reader) {
                    if let Some(handle) = &self.stream_handle {
                        if let Ok(sink) = Sink::try_new(handle) {
                            sink.set_volume(self.effective_volume());
                            let skipped = decoder.skip_duration(pos);
                            sink.append(skipped);
                            sink.play();
                            self.sink = Some(sink);
                            self.seek_offset = pos;
                            self.start_time = Some(Instant::now());
                            self.is_paused.store(false, Ordering::SeqCst);
                        }
                    }
                }
            }
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if self.muted && self.volume > 0.0 {
            self.muted = false;
        }
        if let Some(sink) = &self.sink {
            sink.set_volume(self.effective_volume());
        }
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.muted = !self.muted;
        let eff = self.effective_volume();
        if let Some(sink) = &self.sink {
            sink.set_volume(eff);
        }
        self.muted
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn is_playing(&self) -> bool {
        if let Some(sink) = &self.sink {
            !sink.is_paused() && !sink.empty()
        } else {
            false
        }
    }

    pub fn is_finished(&self) -> bool {
        if let Some(sink) = &self.sink {
            sink.empty()
        } else {
            false
        }
    }

    /// Calculate pseudo-level VU meter values (0.0 to 1.0) for Left and Right channels when audio is playing
    pub fn get_levels(&self) -> (f32, f32) {
        if !self.is_playing() {
            return (0.0, 0.0);
        }

        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs_f32();
            let vol = self.effective_volume();
            
            // Dynamic LED VU visualization simulation based on audio playback harmonics
            let l = ((elapsed * 12.0).sin().abs() * 0.7 + (elapsed * 23.0).cos().abs() * 0.3) * vol;
            let r = ((elapsed * 15.0).cos().abs() * 0.7 + (elapsed * 19.0).sin().abs() * 0.3) * vol;
            (l.clamp(0.05, 1.0), r.clamp(0.05, 1.0))
        } else {
            (0.0, 0.0)
        }
    }
}

fn get_mp3_duration_robust(path: &Path) -> Option<Duration> {
    let file = File::open(path).ok()?;
    let file_len = file.metadata().ok()?.len();
    if file_len == 0 {
        return None;
    }

    let data = std::fs::read(path).ok()?;
    if data.len() < 4 {
        return None;
    }

    let mut offset = 0;
    // Skip ID3v2 header if present
    if data.len() >= 10 && &data[0..3] == b"ID3" {
        let size = ((data[6] as usize & 0x7F) << 21)
            | ((data[7] as usize & 0x7F) << 14)
            | ((data[8] as usize & 0x7F) << 7)
            | (data[9] as usize & 0x7F);
        offset = 10 + size;
        let flags = data[5];
        if (flags & 0x10) != 0 {
            // Footer present
            offset += 10;
        }
    }

    let mut total_samples: u64 = 0;
    let mut sample_rate: u32 = 0;
    let mut first_bitrate: Option<u32> = None;

    const BITRATES_V1_L3: [u32; 16] = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0];
    const BITRATES_V2_L3: [u32; 16] = [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0];
    const SAMPLE_RATES_V1: [u32; 4] = [44100, 48000, 32000, 0];
    const SAMPLE_RATES_V2: [u32; 4] = [22050, 24000, 16000, 0];
    const SAMPLE_RATES_V25: [u32; 4] = [11025, 12000, 8000, 0];

    while offset + 4 <= data.len() {
        let b0 = data[offset];
        let b1 = data[offset + 1];
        let b2 = data[offset + 2];

        // Check frame sync (0xFF and top 3 bits of b1 are 111 => 0xE0)
        if b0 == 0xFF && (b1 & 0xE0) == 0xE0 {
            let mpeg_version = (b1 >> 3) & 0x03; // 0=2.5, 2=2, 3=1
            let layer = (b1 >> 1) & 0x03; // 1=L3, 2=L2, 3=L1
            let bitrate_idx = ((b2 >> 4) & 0x0F) as usize;
            let sample_rate_idx = ((b2 >> 2) & 0x03) as usize;
            let padding = ((b2 >> 1) & 0x01) as usize;

            if mpeg_version != 1 && layer == 1 && bitrate_idx != 0 && bitrate_idx != 15 && sample_rate_idx != 3 {
                let s_rate = match mpeg_version {
                    3 => SAMPLE_RATES_V1[sample_rate_idx],
                    2 => SAMPLE_RATES_V2[sample_rate_idx],
                    0 => SAMPLE_RATES_V25[sample_rate_idx],
                    _ => 0,
                };

                let bitrate_kbps = match mpeg_version {
                    3 => BITRATES_V1_L3[bitrate_idx],
                    _ => BITRATES_V2_L3[bitrate_idx],
                };

                if s_rate > 0 && bitrate_kbps > 0 {
                    let samples_per_frame = if mpeg_version == 3 { 1152 } else { 576 };
                    let frame_len = (samples_per_frame as usize * bitrate_kbps as usize * 1000 / 8) / s_rate as usize + padding;

                    if frame_len > 0 && offset + frame_len <= data.len() {
                        total_samples += samples_per_frame as u64;
                        sample_rate = s_rate;
                        if first_bitrate.is_none() {
                            first_bitrate = Some(bitrate_kbps);
                        }
                        offset += frame_len;
                        continue;
                    }
                }
            }
        }

        // Sync lost or invalid frame header, advance by 1 byte to find next sync
        offset += 1;
    }

    if sample_rate > 0 && total_samples > 0 {
        let secs = total_samples as f64 / sample_rate as f64;
        Some(Duration::from_secs_f64(secs))
    } else if let Some(bitrate) = first_bitrate {
        let secs = (file_len as f64 * 8.0) / (bitrate as f64 * 1000.0);
        if secs > 0.0 {
            Some(Duration::from_secs_f64(secs))
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_mp3_duration_robust() {
        let dir = PathBuf::from("/var/mnt/THRASHER/downloads/torrents/Xanth Series Books 1 - 32/Piers Anthony - Xanth - 01 - A Spell For Chameleon");
        if !dir.exists() { return; }
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) == Some("mp3") {
                let custom_dur = get_mp3_duration_robust(&path);
                assert!(custom_dur.is_some(), "Duration should be detected for {:?}", path);
                assert!(custom_dur.unwrap().as_secs() > 0);
            }
        }
    }
}

