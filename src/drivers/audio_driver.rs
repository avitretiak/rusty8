use sdl2::audio::{AudioCallback, AudioDevice};
use std::f32::consts::PI;

const AMPLITUDE: f32 = 0.25;
const SAMPLE_RATE: i32 = 44100;
const ATTACK_TIME: f32 = 0.01; // 10ms attack
const DECAY_TIME: f32 = 0.05; // 50ms decay

pub struct SineWave {
    phase: f32,
    volume: f32,
    frequency: f32,
    sample_count: usize,
    duration: usize,
}

impl SineWave {
    pub fn new(frequency: f32) -> Self {
        SineWave {
            phase: 0.0,
            volume: AMPLITUDE,
            frequency,
            sample_count: 0,
            duration: 0,
        }
    }

    pub fn set_duration(&mut self, duration_ms: u32) {
        self.duration = (duration_ms as f32 * SAMPLE_RATE as f32 / 1000.0) as usize;
        self.sample_count = 0;
    }
}

impl AudioCallback for SineWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            let t = self.sample_count as f32 / SAMPLE_RATE as f32;
            let envelope = if t < ATTACK_TIME {
                t / ATTACK_TIME
            } else if t < ATTACK_TIME + DECAY_TIME {
                1.0 - (t - ATTACK_TIME) / DECAY_TIME * 0.1
            } else {
                0.9
            };

            *x = (self.phase * 2.0 * PI).sin() * self.volume * envelope;
            self.phase = (self.phase + self.frequency / SAMPLE_RATE as f32) % 1.0;

            self.sample_count += 1;
            if self.sample_count >= self.duration {
                *x = 0.0;
            }
        }
    }
}

pub fn play_beep(device: &mut AudioDevice<SineWave>, duration: u32) {
    device.lock().set_duration(duration);
    device.resume();
}

pub fn stop_beep(device: &mut AudioDevice<SineWave>) {
    device.pause();
}
