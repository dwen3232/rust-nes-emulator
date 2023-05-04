use std::fs::{OpenOptions, remove_file, read_to_string};
use std::io::Write;

use rust_nes_emulator::{
    cpu::CPU,
    trace::trace_cpu,
    traits::Memory
};


#[test]
fn test_cpu_initialization() {
    // Tests that the cpu initializes properly,
    let cpu = CPU::new_empty();
    assert_eq!(0, cpu.reg_a);
}


#[test]
fn test_cpu_official_opcodes_nestest() {
    // Tests only the official opcodes
    remove_file("logs/test_cpu_nestest.log").err();

    let mut f = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("logs/test_cpu_nestest.log")
        .unwrap();


    let mut cpu = CPU::new_empty();
    let mut result: Vec<String> = vec![];
    cpu.run_nes_with_callback(
        "test_roms/nestest.nes",
        |cpu| { 
            if let Ok(s) = trace_cpu(cpu) {
                writeln!(f, "{}", s).expect("Couldn't write line");
                result.push(s);
            } else {

            }
        }
    ).expect_err("Expected to fail");

    let expected_log: Vec<String> = read_to_string("logs/nestest.log")
        .expect("Failed to read input")
        .split("\n")
        .map(|s| s.trim_end().to_string())
        .collect();

    for i in 0..5002 {
        assert_eq!(result[i], expected_log[i], "Diff at line {}", i);
    }

    assert_eq!(cpu.read_byte(0x600), 0);
}