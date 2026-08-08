use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct AudioPlayer {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    sink: Option<Sink>,
    volume: f32,
    temp_muted: bool,
    start_time: Option<Instant>,
    playing_path: Option<String>,
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
            temp_muted: false,
            start_time: None,
            playing_path: None,
            is_paused: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn play(&mut self, path: &Path) -> Result<(), String> {
        self.stop();

        let handle = match &self.stream_handle {
            Some(h) => h,
            None => return Err("Audio device not available".to_string()),
        };

        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader).map_err(|e| format!("Failed to decode audio: {}", e))?;

        let sink = Sink::try_new(handle).map_err(|e| format!("Failed to create sink: {}", e))?;
        sink.set_volume(if self.temp_muted { self.volume * 0.7 } else { self.volume });
        sink.append(decoder);
        sink.play();

        self.sink = Some(sink);
        self.start_time = Some(Instant::now());
        self.playing_path = Some(path.to_string_lossy().to_string());
        self.is_paused.store(false, Ordering::SeqCst);

        Ok(())
    }

    pub fn toggle_pause(&mut self) {
        if let Some(sink) = &self.sink {
            if sink.is_paused() {
                sink.play();
                self.is_paused.store(false, Ordering::SeqCst);
            } else {
                sink.pause();
                self.is_paused.store(true, Ordering::SeqCst);
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.start_time = None;
        self.playing_path = None;
        self.is_paused.store(false, Ordering::SeqCst);
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(sink) = &self.sink {
            sink.set_volume(if self.temp_muted { self.volume * 0.7 } else { self.volume });
        }
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn toggle_temp_mute(&mut self) {
        self.temp_muted = !self.temp_muted;
        let effective = if self.temp_muted { self.volume * 0.7 } else { self.volume };
        if let Some(sink) = &self.sink {
            sink.set_volume(effective);
        }
    }

    pub fn is_temp_muted(&self) -> bool {
        self.temp_muted
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
            let vol = if self.temp_muted { self.volume * 0.7 } else { self.volume };
            
            // Dynamic LED VU visualization simulation based on audio playback harmonics
            let l = ((elapsed * 12.0).sin().abs() * 0.7 + (elapsed * 23.0).cos().abs() * 0.3) * vol;
            let r = ((elapsed * 15.0).cos().abs() * 0.7 + (elapsed * 19.0).sin().abs() * 0.3) * vol;
            (l.clamp(0.05, 1.0), r.clamp(0.05, 1.0))
        } else {
            (0.0, 0.0)
        }
    }
}
