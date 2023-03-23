use rust_nes_emulator::cpu::CPU;

#[test]
fn test_cpu_initialization() {
    // Tests that the cpu initializes properly,
    let cpu = CPU::new();
    assert_eq!(0, cpu.reg_a);
}
