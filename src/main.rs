 use sdl2::event::Event;
 use sdl2::keyboard::Keycode;
 use sdl2::pixels::Color;
 use sdl2::rect::Rect;
 use sdl2::render::Canvas;
 use sdl2::video::Window;
 use std::time::Duration;

 mod drivers;
 mod processor;

 use drivers::{audio_driver, cartridge_driver, display_driver, input_driver};
 use processor::CPU;

 fn main() -> Result<(), String> {
     let sdl_context = sdl2::init()?;
     let video_subsystem = sdl_context.video()?;
     let window = video_subsystem.window("CHIP-8 Emulator", 640, 320)
         .position_centered()
         .build()
         .map_err(|e| e.to_string())?;

     let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
     let mut event_pump = sdl_context.event_pump()?;

     let mut cpu = CPU::new();
     let rom_data = cartridge_driver::load_rom("test_opcode.ch8")?;
     cpu.load_rom(&rom_data);

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

         cpu.tick();
         cpu.tick_60hz();

         if cpu.renderer.redraw {
             display_driver::update_display(&mut canvas, &cpu.renderer.buffer)?;
             cpu.renderer.redraw = false;
         }

         ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
     }

     Ok(())
 }