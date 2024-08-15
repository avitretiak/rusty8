use drivers::input_driver::InputDriver;
use sdl2::audio::{AudioDevice, AudioSpecDesired};
use std::env;
use std::time::{Duration, Instant};

mod config;
mod drivers;
mod processor;

use config::Config;
use drivers::{audio_driver, cartridge_driver, display_driver};
use processor::CPU;

const CHIP8_WIDTH: u32 = 64;
const CHIP8_HEIGHT: u32 = 32;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let event_pump = sdl_context.event_pump()?;
    let mut input_driver = InputDriver::new(event_pump);
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err("Usage: cargo run <path_to_rom>".to_string());
    }
    let rom_path = &args[1];

    let sdl_context = sdl2::init().map_err(|e| e.to_string())?;
    let video_subsystem = sdl_context.video()?;
    let scale_factor = 10 * 2; // Default scale factor
    let window = video_subsystem
        .window(
            "CHIP-8 Emulator",
            CHIP8_WIDTH * scale_factor,
            CHIP8_HEIGHT * scale_factor,
        )
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let audio_subsystem = sdl_context.audio().map_err(|e| e.to_string())?;

    let audio_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };
    let mut audio_device: AudioDevice<audio_driver::SineWave> = audio_subsystem
        .open_playback(None, &audio_spec, |_spec| {
            audio_driver::SineWave::new(800.0)
        })
        .map_err(|e| e.to_string())?;

    let mut cpu = CPU::new();
    let rom_data = cartridge_driver::load_rom(rom_path)?;
    cpu.load_rom(&rom_data);

    let config = Config::new(scale_factor);

    let mut last_sound_time = Instant::now();
    let mut last_tick_time = Instant::now();
    let mut beep_start_time: Option<Instant> = None;

    'running: loop {
        let keypad = match input_driver.poll() {
            Ok(keypad) => keypad,
            Err(_) => break 'running,
        };

        cpu.tick(keypad);

        let now = Instant::now();
        if now.duration_since(last_tick_time) >= Duration::from_micros(1000000 / 500) {
            cpu.tick(keypad);
            last_tick_time = now;
        }

        if now.duration_since(last_sound_time) >= Duration::from_millis(1000 / 60) {
            cpu.tick_60hz();
            last_sound_time = now;
        }

        if cpu.renderer.redraw {
            display_driver::update_display(&mut canvas, &cpu.renderer.buffer, &config)?;
            cpu.renderer.redraw = false;
        }

        if cpu.sound_timer > 0 {
            if beep_start_time.is_none() {
                audio_driver::play_beep(&mut audio_device, 100);
                beep_start_time = Some(Instant::now());
            }
        } else if let Some(start_time) = beep_start_time {
            if now.duration_since(start_time) >= Duration::from_millis(100) {
                audio_driver::stop_beep(&mut audio_device);
                beep_start_time = None;
            }
        }

        let sleep_duration = Duration::from_micros(1000000 / 500);
        ::std::thread::sleep(sleep_duration);
    }

    Ok(())
}

#[cfg(test)]
mod processor_test;
