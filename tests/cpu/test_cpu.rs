use std::fs::{OpenOptions, remove_file, read_to_string};
use std::io::Write;

use rust_nes_emulator::cpu::Opcode;
use rust_nes_emulator::nes::{ActionNES, NES};



#[test]
fn test_cpu_official_opcodes_nestest() {
    // Tests only the official opcodes
    println!("Removing file");
    remove_file("logs/test_cpu_official_opcodes_nestest.log").err();

    let mut f = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("logs/test_cpu_official_opcodes_nestest.log")
        .unwrap();

    println!("Creating ActionNES");
    let mut nes = ActionNES::new();
    println!("Loading from path");
    nes.load_from_path("test_roms/nestest.nes").expect("Failed to load from path");
    nes.cpu_state.program_counter = nes.as_cpu_bus().peek_two_bytes(0xFFFC) - 4;
    for _ in 0..5002 {
        let instruction = nes.next_cpu_instruction().expect("Failed to run instruction");
        if instruction.opcode == Opcode::BRK {
            break;
        }
        if let Some(s) = nes.program_trace.last() {
            writeln!(f, "{}", s).expect("Couldn't write line");
        }
    }
    println!("Ran {:?} instructions", nes.program_trace.len());

    let expected_log: Vec<String> = read_to_string("logs/nestest.log")
        .expect("Failed to read input")
        .split("\n")
        .map(|s| s.trim_end().to_string())
        .collect();

    for i in 0..5002 {
        let trace_line = &nes.program_trace[i];
        let trimmed_line: String = trace_line.chars().take(73).collect();
        assert_eq!(trimmed_line, expected_log[i], "Diff at line {}", i);
    }

    // assert_eq!(cpu.read_byte(0x600), 0);
}

// #[test]
// fn test_cpu_ppu_timings() {
//         // Tests timing of only the official opcodes
//         remove_file("logs/test_cpu_ppu_timings.log").err();

//         let mut f = OpenOptions::new()
//             .write(true)
//             .append(true)
//             .create(true)
//             .open("logs/test_cpu_ppu_timings.log")
//             .unwrap();
    
    
//         let mut cpu = CPU::new_empty();
//         let mut result: Vec<String> = vec![];
//         cpu.load_nes("test_roms/nestest.nes");
//         cpu.program_counter = cpu.read_two_bytes(0xFFFC) - 4;
//         cpu.increment_cycle_counter(7);
//         cpu.run_with_callback(
//             |cpu| { 
//                 if let Ok(s) = trace_cpu(cpu, true) {
//                     writeln!(f, "{}", s).expect("Couldn't write line");
//                     result.push(s);
//                 }
//             }
//         ).unwrap_or_default();
    
//         let expected_log: Vec<String> = read_to_string("logs/nestest_ppu_cyc.log")
//             .expect("Failed to read input")
//             .split("\n")
//             .map(|s| s.trim_end().to_string())
//             .collect();
    
//         for i in 0..5002 {
//             assert_eq!(result[i], expected_log[i], 
//             "\nDiff at line {}\n{}", 
//             i, expected_log[if i > 0 { i - 1 } else { i }]);
//         }
    
//         assert_eq!(cpu.read_byte(0x600), 0);
// }