use sdl2::keyboard::Keycode;
use crate::processor::CPU;

pub fn handle_key_down(cpu: &mut CPU, key: Keycode) {
    if let Some(chip8_key) = map_keycode_to_chip8(key) {
        cpu.handle_key_event(chip8_key, true);
    }
}

pub fn handle_key_up(cpu: &mut CPU, key: Keycode) {
    if let Some(chip8_key) = map_keycode_to_chip8(key) {
        cpu.handle_key_event(chip8_key, false);
    }
}

fn map_keycode_to_chip8(key: Keycode) -> Option<u8> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
