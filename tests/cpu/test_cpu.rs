use std::fs::{remove_file, read_to_string};
use std::{thread, time, fs::OpenOptions};
use std::io::Write;

use rust_nes_emulator::{
    cpu::{CPU, Memory}, 
    trace::trace_cpu
};

#[test]
fn test_cpu_initialization() {
    // Tests that the cpu initializes properly,
    let cpu = CPU::new_empty();
    assert_eq!(0, cpu.reg_a);
}

// #[test]
// fn test_cpu_blargg_instr_test_01_implied() {
//     let mut cpu = CPU::new_empty();
//     cpu.run_nes_with_callback(
//         "test_roms/01-implied.nes",
//         |_| { thread::sleep(time::Duration::from_secs(1)) }
//     ).expect("Expected to run successfully");

//     assert_eq!(cpu.read_byte(0x600), 0);
// }

#[test]
fn test_cpu_nestest() {
    // remove and remake test_cpu_nestest.log
    remove_file("logs/test_cpu_nestest.log");
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
            // thread::sleep(time::Duration::from_secs(1));
            let trace = trace_cpu(cpu);
            writeln!(f, "{}", trace).expect("Couldn't write line");
            result.push(trace);
        }
    );

    let expected_log: Vec<String> = read_to_string("logs/nestest.log")
        .expect("Failed to read input")
        .split("\n")
        .map(|s| s.to_string())
        .collect();

    // this doesn't really work yet because of all the panics I have. Need to refactor to get rid of them
    for i in 0..expected_log.len() {
        assert_eq!(result[i], expected_log[i], "Diff at line {}", i);
    }

    // assert_eq!(cpu.read_byte(0x600), 0);
}