use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::env;
use std::time::{Duration, Instant};
use sdl2::audio::{AudioSpecDesired, AudioDevice};

mod drivers;
mod processor;

use drivers::{audio_driver, cartridge_driver, display_driver, input_driver};
use processor::CPU;


const CHIP8_WIDTH: u32 = 64;
const CHIP8_HEIGHT: u32 = 32;


fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err("Usage: cargo run <path_to_rom>".to_string());
    }
    let rom_path = &args[1];

    let sdl_context = sdl2::init().map_err(|e| e.to_string())?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem.window("CHIP-8 Emulator", CHIP8_WIDTH * 10, CHIP8_HEIGHT * 10)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;
    let audio_subsystem = sdl_context.audio().map_err(|e| e.to_string())?;

    // Create the AudioDevice here
    let audio_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };
    let mut audio_device: AudioDevice<audio_driver::SineWave> = audio_subsystem.open_playback(None, &audio_spec, |_spec| {
        audio_driver::SineWave::new(800.0) // Use the beep frequency
    }).map_err(|e| e.to_string())?;

    let mut cpu = CPU::new();
    let rom_data = cartridge_driver::load_rom(rom_path)?;
    cpu.load_rom(&rom_data);

    let mut last_sound_time = Instant::now();
    let mut last_tick_time = Instant::now();
    let mut beep_start_time: Option<Instant> = None;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(key), .. } => {
                    input_driver::handle_key_down(&mut cpu, key);
                },
                Event::KeyUp { keycode: Some(key), .. } => {
                    input_driver::handle_key_up(&mut cpu, key);
                },
                _ => {}
            }
        }

        let now = Instant::now();
        if now.duration_since(last_tick_time) >= Duration::from_micros(1000000 / 500) {
            cpu.tick();
            last_tick_time = now;
        }

        if now.duration_since(last_sound_time) >= Duration::from_millis(1000 / 60) {
            cpu.tick_60hz();
            last_sound_time = now;
        }

        if cpu.renderer.redraw {
            display_driver::update_display(&mut canvas, &cpu.renderer.buffer)?;
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

        // Use a more precise timing mechanism
        let sleep_duration = Duration::from_micros(1000000 / 500);
        ::std::thread::sleep(sleep_duration);
    }

    Ok(())
}


