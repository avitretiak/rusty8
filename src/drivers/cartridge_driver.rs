use std::fs::File;
use std::io::Read;

pub fn load_rom(path: &str) -> Result<Vec<u8>, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut rom_data = Vec::new();
    file.read_to_end(&mut rom_data).map_err(|e| e.to_string())?;
    Ok(rom_data)
}
