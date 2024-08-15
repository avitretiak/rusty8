use rand::Rng;

pub struct CPU {
    pub keypad: [bool; 16],
    pub memory: [u8; 4096],
    pub registers: [u8; 16],
    pub index: u16,
    pub program_counter: u16,
    pub stack: [u16; 16],
    pub stack_pointer: u8,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub renderer: Renderer,
    random: rand::rngs::StdRng,
    waiting_for_key: Option<usize>,
}

pub struct Renderer {
    pub buffer: [[bool; 64]; 32],
    pub redraw: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            keypad: [false; 16],
            memory: [0; 4096],
            registers: [0; 16],
            index: 0,
            program_counter: 0x200,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            renderer: Renderer {
                buffer: [[false; 64]; 32],
                redraw: false,
            },
            random: rand::SeedableRng::from_entropy(),
            waiting_for_key: None,
        }
    }

    pub fn load_rom(&mut self, rom_data: &[u8]) {
        self.memory[0x200..0x200 + rom_data.len()].copy_from_slice(rom_data);
    }

    pub fn tick(&mut self, keypad: [bool; 16]) {
        self.keypad = keypad;
        if let Some(register) = self.waiting_for_key {
            for key in 0..=0xF {
                if self.keypad[key] {
                    self.registers[register] = key as u8;
                    self.waiting_for_key = None;
                    self.program_counter += 2; // Move to the next instruction
                    return;
                }
            }
        } else {
            let opcode: u16 = ((self.memory[self.program_counter as usize] as u16) << 8)
                | self.memory[(self.program_counter + 1) as usize] as u16;

            self.program_counter += 2;
            self.execute_opcode(opcode);
        }
    }

    pub fn tick_60hz(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn execute_opcode(&mut self, opcode: u16) {
        let nibbles = (
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            opcode & 0x000F,
        );

        match nibbles {
            (0x00, 0x00, 0x0e, 0x00) => self.clear_display(),
            (0x00, 0x00, 0x0e, 0x0e) => self.return_from_subroutine(),
            (0x01, _, _, _) => self.jump(opcode),
            (0x02, _, _, _) => self.call(opcode),
            (0x03, _, _, _) => self.skip_if_x_equal(opcode),
            (0x04, _, _, _) => self.skip_if_x_not_equal(opcode),
            (0x05, _, _, 0x00) => self.skip_if_x_and_y_equal(opcode),
            (0x06, _, _, _) => self.set_x(opcode),
            (0x07, _, _, _) => self.add_x(opcode),
            (0x08, _, _, 0x00) => self.arithmetic(opcode),
            (0x08, _, _, 0x01) => self.arithmetic(opcode),
            (0x08, _, _, 0x02) => self.arithmetic(opcode),
            (0x08, _, _, 0x03) => self.arithmetic(opcode),
            (0x08, _, _, 0x04) => self.arithmetic(opcode),
            (0x08, _, _, 0x05) => self.arithmetic(opcode),
            (0x08, _, _, 0x06) => self.arithmetic(opcode),
            (0x08, _, _, 0x07) => self.arithmetic(opcode),
            (0x08, _, _, 0x0e) => self.arithmetic(opcode),
            (0x09, _, _, 0x00) => self.skip_if_x_and_y_different(opcode),
            (0x0a, _, _, _) => self.set_index(opcode),
            (0x0b, _, _, _) => self.jump_with_offset(opcode),
            (0x0c, _, _, _) => self.random(opcode),
            (0x0d, _, _, _) => self.draw_sprite(opcode),
            (0x0e, _, 0x09, 0x0e) => self.skip_if_pressed(opcode),
            (0x0e, _, 0x0a, 0x01) => self.skip_if_not_pressed(opcode),
            (0x0f, _, 0x00, 0x07) => self.misc(opcode),
            (0x0f, _, 0x00, 0x0a) => self.misc(opcode),
            (0x0f, _, 0x01, 0x05) => self.misc(opcode),
            (0x0f, _, 0x01, 0x08) => self.misc(opcode),
            (0x0f, _, 0x01, 0x0e) => self.misc(opcode),
            (0x0f, _, 0x02, 0x09) => self.misc(opcode),
            (0x0f, _, 0x03, 0x03) => self.misc(opcode),
            (0x0f, _, 0x05, 0x05) => self.misc(opcode),
            (0x0f, _, 0x06, 0x05) => self.misc(opcode),
            _ => (),
        }
    }

    fn clear_display(&mut self) {
        self.renderer.buffer = [[false; 64]; 32];
        self.renderer.redraw = true;
    }

    fn return_from_subroutine(&mut self) {
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
    }

    fn jump(&mut self, opcode: u16) {
        self.program_counter = opcode & 0x0FFF;
    }

    fn call(&mut self, opcode: u16) {
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = opcode & 0x0FFF;
    }

    fn skip_if_x_equal(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        if self.registers[x] == nn {
            self.program_counter += 2;
        }
    }

    fn skip_if_x_not_equal(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        if self.registers[x] != nn {
            self.program_counter += 2;
        }
    }

    fn skip_if_x_and_y_equal(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.registers[x] == self.registers[y] {
            self.program_counter += 2;
        }
    }

    fn set_x(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        self.registers[x] = nn;
    }

    fn add_x(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        self.registers[x] = self.registers[x].wrapping_add(nn);
    }

    fn arithmetic(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as u8;

        match n {
            0x0 => self.registers[x] = self.registers[y],
            0x1 => self.registers[x] |= self.registers[y],
            0x2 => self.registers[x] &= self.registers[y],
            0x3 => self.registers[x] ^= self.registers[y],
            0x4 => {
                let (result, overflow) = self.registers[x].overflowing_add(self.registers[y]);
                self.registers[x] = result;
                self.registers[0xF] = if overflow { 1 } else { 0 };
            }
            0x5 => {
                let vx = self.registers[x];
                let vy = self.registers[y];
                self.registers[x] = vx.wrapping_sub(vy);
                self.registers[0xF] = if vx >= vy { 1 } else { 0 };
            }
            0x6 => {
                let lsb = self.registers[x] & 0x1;
                self.registers[x] >>= 1;
                self.registers[0xF] = lsb;
            }
            0x7 => {
                let vx = self.registers[x];
                let vy = self.registers[y];
                self.registers[x] = vy.wrapping_sub(vx);
                self.registers[0xF] = if vy >= vx { 1 } else { 0 };
            }
            0xE => {
                let msb = (self.registers[x] >> 7) & 0x1;
                self.registers[x] <<= 1;
                self.registers[0xF] = msb;
            }
            _ => (),
        }
    }

    fn skip_if_x_and_y_different(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.registers[x] != self.registers[y] {
            self.program_counter += 2;
        }
    }

    fn set_index(&mut self, opcode: u16) {
        self.index = opcode & 0x0FFF;
    }

    fn jump_with_offset(&mut self, opcode: u16) {
        self.program_counter = (opcode & 0x0FFF) + self.registers[0] as u16;
    }

    fn random(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        self.registers[x] = self.random.gen::<u8>() & nn;
    }

    fn draw_sprite(&mut self, opcode: u16) {
        let x = self.registers[((opcode & 0x0F00) >> 8) as usize] as usize;
        let y = self.registers[((opcode & 0x00F0) >> 4) as usize] as usize;
        let n = (opcode & 0x000F) as usize;

        self.registers[0xF] = 0;

        let renderer = &mut self.renderer;
        for row in 0..n {
            let sprite_byte = self.memory[(self.index + row as u16) as usize];
            for bit in 0..8 {
                let sprite_bit = (sprite_byte >> (7 - bit)) & 1;
                let buffer_x = (x + bit) % 64;
                let buffer_y = (y + row) % 32;

                if sprite_bit == 1 {
                    if renderer.buffer[buffer_y][buffer_x] {
                        self.registers[0xF] = 1;
                    }
                    renderer.buffer[buffer_y][buffer_x] ^= true;
                }
            }
        }

        renderer.redraw = true;
    }

    fn skip_if_pressed(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        if self.keypad[self.registers[x] as usize] {
            self.program_counter += 2;
        }
    }

    fn skip_if_not_pressed(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        if !self.keypad[self.registers[x] as usize] {
            self.program_counter += 2;
        }
    }

    fn wait_for_key_press(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        if let Some(key) = self.keypad.iter().position(|&k| k) {
            self.registers[x] = key as u8;
        } else {
            self.program_counter -= 2; // Repeat this instruction until a key is pressed
        }
    }

    fn misc(&mut self, opcode: u16) {
        let nn = (opcode & 0x00FF) as u8;

        match nn {
            0x07 => self.set_x_to_delay(opcode),
            0x0A => self.wait_for_key_press(opcode),
            0x15 => self.set_delay_to_x(opcode),
            0x18 => self.set_sound_to_x(opcode),
            0x1E => self.add_x_to_index(opcode),
            0x29 => self.set_index_for_char(opcode),
            0x33 => self.binary_coded_decimal(opcode),
            0x55 => self.save_x(opcode),
            0x65 => self.load_x(opcode),
            _ => (),
        }
    }

    fn set_x_to_delay(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.registers[x] = self.delay_timer;
    }

    fn set_delay_to_x(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.delay_timer = self.registers[x];
    }

    fn set_sound_to_x(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.sound_timer = self.registers[x];
    }

    fn add_x_to_index(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let (result, _) = self.index.overflowing_add(self.registers[x] as u16);
        self.index = result;
    }

    fn set_index_for_char(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.index = (self.registers[x] as u16) * 5;
    }

    fn binary_coded_decimal(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let value = self.registers[x];
        self.memory[self.index as usize] = value / 100;
        self.memory[(self.index + 1) as usize] = (value / 10) % 10;
        self.memory[(self.index + 2) as usize] = value % 10;
    }

    fn save_x(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        for i in 0..=x {
            self.memory[(self.index + i as u16) as usize] = self.registers[i];
        }
    }

    fn load_x(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        for i in 0..=x {
            self.registers[i] = self.memory[(self.index + i as u16) as usize];
        }
    }
}
