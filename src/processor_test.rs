#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_initialization() {
        let cpu = CPU::new();
        assert_eq!(cpu.program_counter, 0x200);
        // Add more assertions as needed
    }

    // Add more tests as needed
}
