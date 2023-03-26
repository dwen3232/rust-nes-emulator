use std::{thread, time};


use rust_nes_emulator::cpu::{CPU, Memory};

#[test]
fn test_cpu_initialization() {
    // Tests that the cpu initializes properly,
    let cpu = CPU::new();
    assert_eq!(0, cpu.reg_a);
}

#[test]
fn test_cpu_blargg_instr_test_01_implied() {
    let mut cpu = CPU::new();
    cpu.run_nes_with_callback(
        "test_roms/01-implied.nes",
        |_| { thread::sleep(time::Duration::from_secs(1)) }
    ).expect("Expected to run successfully");

    assert_eq!(cpu.read_byte(0x600), 0);
}

#[test]
fn test_cpu_nestest() {
    let mut cpu = CPU::new();
    cpu.run_nes_with_callback(
        "test_roms/nestest.nes",
        |cpu| { 
            thread::sleep(time::Duration::from_secs(1));
            println!("Found {:x} at {:x}", cpu.read_two_bytes(0xFFFC), 0xFFFC);
        }
    ).expect("Expected to run successfully");

    assert_eq!(cpu.read_byte(0x600), 0);
}