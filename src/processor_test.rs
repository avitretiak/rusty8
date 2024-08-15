#[cfg(test)]
use crate::processor::CPU;

#[test]
fn test_00e0_clear_display() {
    let mut cpu = CPU::new();
    cpu.renderer.buffer[0][0] = true;
    cpu.renderer.buffer[31][63] = true;
    cpu.execute_opcode(0x00E0); // Clear display
    assert!(cpu
        .renderer
        .buffer
        .iter()
        .all(|row| row.iter().all(|&pixel| !pixel)));
    assert!(cpu.renderer.redraw);
}

#[test]
fn test_dxyn_draw_sprite() {
    let mut cpu = CPU::new();
    cpu.index = 0x300;
    cpu.memory[0x300] = 0b10101010;
    cpu.memory[0x301] = 0b01010101;
    cpu.registers[0] = 0; // X coordinate
    cpu.registers[1] = 0; // Y coordinate
    cpu.execute_opcode(0xD012); // Draw 2-byte sprite at (0, 0)
    assert_eq!(
        cpu.renderer.buffer[0],
        [
            true, false, true, false, true, false, true, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false
        ]
    );
    assert_eq!(
        cpu.renderer.buffer[1],
        [
            false, true, false, true, false, true, false, true, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false
        ]
    );
    assert!(cpu.renderer.redraw);
}

#[test]
fn test_3x_skip_if_equal() {
    let mut cpu = CPU::new();
    cpu.registers[6] = 0x43;
    cpu.execute_opcode(0x3643); // Skip if V6 == 0x43
    assert_eq!(cpu.program_counter, 0x202); // PC should be incremented by 2 if condition is met
}

#[test]
fn test_4x_skip_if_not_equal() {
    let mut cpu = CPU::new();
    cpu.registers[5] = 0x42;
    cpu.execute_opcode(0x4543); // Skip if Vs5 != 0x43
    assert_eq!(cpu.program_counter, 0x202);
}

#[test]
fn test_5x_skip_if_equal() {
    let mut cpu = CPU::new();
    cpu.registers[5] = 0x42;
    cpu.registers[6] = 0x42;
    cpu.execute_opcode(0x5560); // Skip if V5 == V6
    assert_eq!(cpu.program_counter, 0x202);
}

#[test]
fn test_7x_add() {
    let mut cpu = CPU::new();
    cpu.registers[6] = 255;
    cpu.execute_opcode(0x7601); // V6 += 1
    assert_eq!(cpu.registers[6], 0); // Should wrap around
}

#[test]
fn test_9x_skip_if_not_equal() {
    let mut cpu = CPU::new();
    cpu.registers[5] = 42;
    cpu.registers[6] = 43;
    cpu.execute_opcode(0x9560); // Skip if V5 != V6
    assert_eq!(cpu.program_counter, 0x202);
}

#[test]
fn test_ax_set_index() {
    let mut cpu = CPU::new();
    cpu.execute_opcode(0xA123); // I = 0x123
    assert_eq!(cpu.index, 0x123);
}

#[test]
fn test_8xy0_set() {
    let mut cpu = CPU::new();
    cpu.registers[5] = 42;
    cpu.execute_opcode(0x8750); // V7 = V5
    assert_eq!(cpu.registers[7], 42);
}

#[test]
fn test_8xy1_or() {
    let mut cpu = CPU::new();
    cpu.registers[7] = 0b1010;
    cpu.registers[1] = 0b0101;
    cpu.execute_opcode(0x8711); // V7 |= V1
    assert_eq!(cpu.registers[7], 0b1111);
}

#[test]
fn test_8xy2_and() {
    let mut cpu = CPU::new();
    cpu.registers[6] = 0b1100;
    cpu.registers[7] = 0b1010;
    cpu.execute_opcode(0x8672); // V6 &= V7
    assert_eq!(cpu.registers[6], 0b1000);
}

#[test]
fn test_8xy3_xor() {
    let mut cpu = CPU::new();
    cpu.registers[6] = 0b1100;
    cpu.registers[7] = 0b1010;
    cpu.execute_opcode(0x8673); // V6 ^= V7
    assert_eq!(cpu.registers[6], 0b0110);
}

#[test]
fn test_8xy4_add_with_carry() {
    let mut cpu = CPU::new();
    cpu.registers[7] = 200;
    cpu.registers[6] = 100;
    cpu.execute_opcode(0x8764); // V7 += V6
    assert_eq!(cpu.registers[7], 44);
    assert_eq!(cpu.registers[0xF], 1); // Carry flag
}

#[test]
fn test_8xy5_sub() {
    let mut cpu = CPU::new();
    cpu.registers[7] = 10;
    cpu.registers[6] = 5;
    cpu.execute_opcode(0x8765); // V7 -= V6
    assert_eq!(cpu.registers[7], 5);
    assert_eq!(cpu.registers[0xF], 1); // No borrow
}

#[test]
fn test_8xy6_shift_right() {
    let mut cpu = CPU::new();
    cpu.registers[6] = 0b11010110;
    cpu.execute_opcode(0x8606); // V6 = V6 SHR 1
    assert_eq!(cpu.registers[6], 0b01101011);
    assert_eq!(cpu.registers[0xF], 0);

    cpu.registers[6] = 0b11010111;
    cpu.execute_opcode(0x8606); // V6 = V6 SHR 1
    assert_eq!(cpu.registers[6], 0b01101011);
    assert_eq!(cpu.registers[0xF], 1);
}

#[test]
fn test_8xye_shift_left() {
    let mut cpu = CPU::new();
    cpu.registers[6] = 0b11010110;
    cpu.execute_opcode(0x860E); // V6 = V6 SHL 1
    assert_eq!(cpu.registers[6], 0b10101100);
    assert_eq!(cpu.registers[0xF], 1);

    cpu.registers[6] = 0b01010110;
    cpu.execute_opcode(0x860E); // V6 = V6 SHL 1
    assert_eq!(cpu.registers[6], 0b10101100);
    assert_eq!(cpu.registers[0xF], 0);
}

#[test]
fn test_fx55_fx65_save_load_registers() {
    let mut cpu = CPU::new();
    cpu.registers[0] = 10;
    cpu.registers[1] = 20;
    cpu.registers[2] = 30;
    cpu.index = 0x300;
    cpu.execute_opcode(0xF255); // Store registers V0 through V2 in memory starting at location I
    assert_eq!(cpu.memory[0x300], 10);
    assert_eq!(cpu.memory[0x301], 20);
    assert_eq!(cpu.memory[0x302], 30);
    assert_eq!(cpu.index, 0x300); // I should not change after operation

    cpu.registers[0] = 0;
    cpu.registers[1] = 0;
    cpu.registers[2] = 0;
    cpu.index = 0x300;
    cpu.execute_opcode(0xF265); // Read registers V0 through V2 from memory starting at location I
    assert_eq!(cpu.registers[0], 10);
    assert_eq!(cpu.registers[1], 20);
    assert_eq!(cpu.registers[2], 30);
    assert_eq!(cpu.index, 0x300); // I should not change after operation
}

#[test]
fn test_fx33_binary_coded_decimal() {
    let mut cpu = CPU::new();
    cpu.registers[2] = 137;
    cpu.index = 0x300;
    cpu.execute_opcode(0xF233); // Store BCD of V2
    assert_eq!(cpu.memory[0x300], 1);
    assert_eq!(cpu.memory[0x301], 3);
    assert_eq!(cpu.memory[0x302], 7);
}
