use rand::Rng;
 use std::collections::HashSet;

 pub struct CPU {
     pub memory: [u8; 4096],
     pub registers: [u8; 16],
     pub index: u16,
     pub program_counter: u16,
     pub stack: [u16; 16],
     pub stack_pointer: u8,
     pub delay_timer: u8,
     pub sound_timer: u8,
     pub renderer: Renderer,
     pub pressed_keys: HashSet<u8>,
     random: rand::rngs::StdRng,
 }

 pub struct Renderer {
     pub buffer: [[bool; 64]; 32],
     pub redraw: bool,
 }

 impl CPU {
     pub fn new() -> Self {
         CPU {
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
             pressed_keys: HashSet::new(),
             random: rand::SeedableRng::from_entropy(),
         }
     }

     pub fn load_rom(&mut self, rom_data: &[u8]) {
         self.memory[0x200..0x200 + rom_data.len()].copy_from_slice(rom_data);
     }

     pub fn tick(&mut self) {
         let opcode: u16 = ((self.memory[self.program_counter as usize] as u16) << 8)
             | self.memory[(self.program_counter + 1) as usize] as u16;

         self.program_counter += 2;
         self.execute_opcode(opcode);
     }

     pub fn tick_60hz(&mut self) {
         if self.delay_timer > 0 {
             self.delay_timer -= 1;
         }
         if self.sound_timer > 0 {
             self.sound_timer -= 1;
         }
     }

     fn execute_opcode(&mut self, opcode: u16) {
         match opcode & 0xF000 {
             0x0000 => match opcode & 0x00FF {
                 0xE0 => self.clear_display(),
                 0xEE => self.return_from_subroutine(),
                 _ => (),
             },
             0x1000 => self.jump(opcode),
             0x2000 => self.call(opcode),
             0x3000 => self.skip_if_x_equal(opcode),
             0x4000 => self.skip_if_x_not_equal(opcode),
             0x5000 => self.skip_if_x_and_y_equal(opcode),
             0x6000 => self.set_x(opcode),
             0x7000 => self.add_x(opcode),
             0x8000 => self.arithmetic(opcode),
             0x9000 => self.skip_if_x_and_y_different(opcode),
             0xA000 => self.set_index(opcode),
             0xB000 => self.jump_with_offset(opcode),
             0xC000 => self.random(opcode),
             0xD000 => self.draw_sprite(opcode),
             0xE000 => self.skip_if_pressed(opcode),
             0xF000 => self.misc(opcode),
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
                 let (result, overflow) = self.registers[x].overflowing_sub(self.registers[y]);
                 self.registers[x] = result;
                 self.registers[0xF] = if overflow { 0 } else { 1 };
             }
             0x6 => {
                 self.registers[0xF] = self.registers[x] & 0x1;
                 self.registers[x] >>= 1;
             }
             0x7 => {
                 let (result, overflow) = self.registers[y].overflowing_sub(self.registers[x]);
                 self.registers[x] = result;
                 self.registers[0xF] = if overflow { 0 } else { 1 };
             }
             0xE => {
                 self.registers[0xF] = (self.registers[x] >> 7) & 0x1;
                 self.registers[x] <<= 1;
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
         let nn = (opcode & 0x00FF) as u8;

         if nn == 0x9E {
             if self.pressed_keys.contains(&self.registers[x]) {
                 self.program_counter += 2;
             }
         } else if nn == 0xA1 {
             if !self.pressed_keys.contains(&self.registers[x]) {
                 self.program_counter += 2;
             }
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

     fn wait_for_key_press(&mut self, opcode: u16) {
         let x = ((opcode & 0x0F00) >> 8) as usize;
         if let Some(&key) = self.pressed_keys.iter().next() {
             self.registers[x] = key;
         } else {
             self.program_counter -= 2;
         }
     }

     fn set_delay_to_x(&mut self, opcode: u16) {
         let x = ((opcode & 0x0F00) >> 8) as usize;
         self.delay_timer = self.registers[x];
     }

     fn set_sound_to_x(&mut self, opcode: u16) {
         let x = ((opcode & 0x0F00) >> 8) as usize;
         // Implement sound timer logic here if needed
     }

     fn add_x_to_index(&mut self, opcode: u16) {
         let x = ((opcode & 0x0F00) >> 8) as usize;
         self.index = self.index.wrapping_add(self.registers[x] as u16);
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

     fn push(&mut self, value: u16) {
         self.stack[self.stack_pointer as usize] = value;
         self.stack_pointer += 1;
     }

     fn pop(&mut self) -> u16 {
         self.stack_pointer -= 1;
         self.stack[self.stack_pointer as usize]
     }

     fn fetch(&self) -> u8 {
         self.memory[self.program_counter as usize]
     }

     pub fn handle_key_event(&mut self, key: u8, is_pressed: bool) {
         if is_pressed {
             self.pressed_keys.insert(key);
         } else {
             self.pressed_keys.remove(&key);
         }
     }
 }