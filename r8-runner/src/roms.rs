use std::io;
use std::io::Read;
use std::fs::File;
use r8_core::Hardware;
use crate::settings::Settings;

pub fn load_from_file(hardware: &mut Hardware, settings: &Settings) -> io::Result<()> {
    // Clear program space in memory
    for x in 512..hardware.memory.len() {
        hardware.memory[x] = 0;
    }

    println!("Loading ROM file: {}", settings.rom_file);

    let mut file = File::open(&settings.rom_file)?;
    let mut buffer = [0; 1024];
    let mut current_memory_index = 512;

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            println!("Rom completely loaded");
            return Ok(());
        }

        for x in 0..bytes_read {
            hardware.memory[current_memory_index + x] = buffer[x];
        }

        current_memory_index += bytes_read;
    }
}