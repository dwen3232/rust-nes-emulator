use std::fs::{read_to_string, remove_file, OpenOptions};
use std::io::Write;

use rust_nes_emulator::cpu::Opcode;
use rust_nes_emulator::tracer::TraceNes;

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
    let mut nes = TraceNes::new().setup();
    println!("Loading from path");
    for _ in 0..5002 {
        let instruction = nes
            .next_cpu_instruction()
            .expect("Failed to run instruction");
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
        .split('\n')
        .map(|s| s.trim_end().to_string())
        .collect();

    for i in 0..5002 {
        let trace_line = &nes.program_trace[i];
        let trimmed_line: String = trace_line.chars().take(73).collect();
        assert_eq!(trimmed_line, expected_log[i], "Diff at line {}", i);
    }

    // assert_eq!(cpu.read_byte(0x600), 0);
}

#[test]
fn test_cpu_official_opcodes_nestest_cycles() {
    // Tests only the official opcodes
    println!("Removing file");
    remove_file("logs/test_cpu_ppu_timings.log").err();

    let mut f = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("logs/test_cpu_ppu_timings.log")
        .unwrap();

    println!("Creating ActionNES");
    let mut nes = TraceNes::new().setup();
    println!("Loading from path");
    for _ in 0..5002 {
        let instruction = nes
            .next_cpu_instruction()
            .expect("Failed to run instruction");
        if instruction.opcode == Opcode::BRK {
            break;
        }
        if let Some(s) = nes.program_trace.last() {
            writeln!(f, "{}", s).expect("Couldn't write line");
        }
    }
    println!("Ran {:?} instructions", nes.program_trace.len());

    let expected_log: Vec<String> = read_to_string("logs/nestest_ppu_cyc.log")
        .expect("Failed to read input")
        .split('\n')
        .map(|s| s.trim_end().to_string())
        .collect();

    for i in 0..5002 {
        let trace_line = nes.program_trace.get(i).expect("Line not found");
        assert_eq!(trace_line, &expected_log[i], "Diff at line {}", i);
    }

    // assert_eq!(cpu.read_byte(0x600), 0);
}
