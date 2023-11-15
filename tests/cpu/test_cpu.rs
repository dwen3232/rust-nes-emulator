use std::fs::{OpenOptions, remove_file, read_to_string};
use std::io::Write;

use rust_nes_emulator::cpu::{Opcode, Instruction, InstructionMetaData, CpuState, CpuBus, AddressingMode, Param};
use rust_nes_emulator::nes::{ActionNES, NES};
use rust_nes_emulator::ppu::PpuState;

type ProgramTrace = Vec<String>;
struct TestNes {
    nes: ActionNES,
    pub program_trace: ProgramTrace,
}

impl TestNes {
    pub fn new() -> Self {
        TestNes {
            nes: ActionNES::new(),
            program_trace: vec![]
        }
    }

    pub fn setup(mut self) -> Self {
        self.nes.load_from_path("test_roms/nestest.nes").expect("Failed to load from path");
        self.nes.cpu_state.program_counter = self.nes.as_cpu_bus().peek_two_bytes(0xFFFC) - 4;
        self.nes.cpu_state.cycle_counter = 7;
        self.nes.ppu_state.cycle_counter = 21;
        self
    }

    pub fn next_cpu_instruction(&mut self) -> Result<Instruction, String> {
        let mut prev_nes = self.nes.clone();
        let instruction = self.nes.next_cpu_instruction()?;
        Self::log_trace(&mut self.program_trace, &instruction, prev_nes)?;
        Ok(instruction)
    }

    /* TODO: this is all spaghetti, need to change this. Maybe move program_trace out of ActionNES
     * and write a wrapper that logs stuff. The logging logic should not be here!
     */
    fn log_trace(
        log: &mut Vec<String>,
        instruction: &Instruction, 
        nes: ActionNES,
    ) -> Result<(), String> {
        let ActionNES { 
            cpu_state: mut original_cpu_state, 
            ppu_state: mut original_ppu_state, 
            controller: mut original_controller, 
            rom,
        } = nes;
        let Instruction { opcode, param, meta } = *instruction;
        let InstructionMetaData { cycles, mode, raw_opcode, length } = meta;

        let mut hex_dump = Vec::new();
        // add opcode byte to dump
        hex_dump.push(raw_opcode);

        let CpuState { reg_a, reg_x, reg_y, status, program_counter, stack_pointer, cycle_counter, ..} = original_cpu_state;
        let cpu_cycle = cycle_counter;

        let PpuState { cur_scanline, cycle_counter, .. } = original_ppu_state;
        let ppu_cycle = cycle_counter;

        // get the parsed arg as a u16
        let arg = match length {
            1 => 0,
            2 => {
                let bus = CpuBus::new(&mut original_cpu_state, &mut original_ppu_state, &mut original_controller, &rom);
                let address: u8 = bus.peek_byte(program_counter + 1);
                hex_dump.push(address);
                address as u16
            },
            3 => {
                let bus = CpuBus::new(&mut original_cpu_state, &mut original_ppu_state, &mut original_controller, &rom);
                let address_lo = bus.peek_byte(program_counter + 1);
                let address_hi = bus.peek_byte(program_counter + 2);
                hex_dump.push(address_lo);
                hex_dump.push(address_hi);

                let address = bus.peek_two_bytes(program_counter + 1);
                address
            },
            _ => {panic!()}
        };

        // create temp string for operand details
        let tmp = match (&instruction, mode, param) {
            // length 1
            (_, AddressingMode::Implicit, _) => String::from(""),
            (_, AddressingMode::Accumulator, _) => {
                format!("A")
            },
            // length 2
            (_, AddressingMode::Immediate, Param::Value(value)) => {
                format!("#${:02x}", value)
            },
            (_, AddressingMode::ZeroPage, Param::Address(address)) => {
                let bus = CpuBus::new(&mut original_cpu_state, &mut original_ppu_state, &mut original_controller, &rom);
                let stored_value = bus.peek_byte(address);
                format!("${:02x} = {:02x}", address, stored_value)
            },
            (_, AddressingMode::ZeroPageIndexX, Param::Address(address)) => {
                let bus = CpuBus::new(&mut original_cpu_state, &mut original_ppu_state, &mut original_controller, &rom);
                let stored_value = bus.peek_byte(address);
                format!(
                    "${:02x},X @ {:02x} = {:02x}",
                    arg, address, stored_value
                )
            },
            (_, AddressingMode::ZeroPageIndexY, Param::Address(address)) => {
                let bus = CpuBus::new(&mut original_cpu_state, &mut original_ppu_state, &mut original_controller, &rom);
                let stored_value = bus.peek_byte(address);
                format!(
                    "${:02x},Y @ {:02x} = {:02x}",
                    arg, address, stored_value
                )
            },
            (_, AddressingMode::IndirectX, Param::Address(address)) => {
                let bus = CpuBus::new(&mut original_cpu_state, &mut original_ppu_state, &mut original_controller, &rom);
                let stored_value = bus.peek_byte(address);
                format!(
                    "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                    arg,
                    (arg.wrapping_add(original_cpu_state.reg_x as u16) as u8),
                    address,
                    stored_value
                )
            },
            (_, AddressingMode::IndirectY, Param::Address(address)) => {
                let bus = CpuBus::new(&mut original_cpu_state, &mut original_ppu_state, &mut original_controller, &rom);
                let stored_value = bus.peek_byte(address);
                format!(
                    "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                    arg,
                    (address.wrapping_sub(original_cpu_state.reg_y as u16)),
                    address,
                    stored_value
                )
            },
            (_, AddressingMode::Relative, _) => {
                let address: usize =
                (program_counter as usize + 2).wrapping_add((arg as i8) as usize);
                format!("${:04x}", address)
            },
            // length 3
            (_, AddressingMode::IndirectJump, Param::Address(address)) => {
                format!("(${:04x}) = {:04x}", arg, address)
            },
            (_, AddressingMode::AbsoluteJump, Param::Address(address)) => {
                format!("${:04x}", address)
            },
            (_, AddressingMode::Absolute, Param::Address(address)) => {
                let bus = CpuBus::new(&mut original_cpu_state, &mut original_ppu_state, &mut original_controller, &rom);
                let stored_value = bus.peek_byte(address);
                format!("${:04x} = {:02x}", address, stored_value)
            },
            (_, AddressingMode::AbsoluteIndexX, Param::Address(address)) => {
                let bus = CpuBus::new(&mut original_cpu_state, &mut original_ppu_state, &mut original_controller, &rom);
                let stored_value = bus.peek_byte(address);
                format!(
                    "${:04x},X @ {:04x} = {:02x}",
                    arg, address, stored_value
                )
            },
            (_, AddressingMode::AbsoluteIndexY, Param::Address(address)) => {
                let bus = CpuBus::new(&mut original_cpu_state, &mut original_ppu_state, &mut original_controller, &rom);
                let stored_value = bus.peek_byte(address);
                format!(
                    "${:04x},Y @ {:04x} = {:02x}",
                    arg, address, stored_value
                )
            },
            (instruction, mode, param) => {
                panic!("Could not trace this argument {:?}, {:?}, {:?}", instruction, mode, param)
            }    
        };
        // Get clock cycle information
        


        // Add strings together
        let opstring = format!("{:?}", opcode);
        let hex_str = hex_dump
            .iter()
            .map(|z| format!("{:02x}", z))
            .collect::<Vec<String>>()
            .join(" ");
        let asm_str = format!("{:04x}  {:8} {: >4} {}", program_counter, hex_str, opstring, tmp)
            .trim()
            .to_string();
        let clock_str = format!(" PPU:{:>3},{:>3} CYC:{}", cur_scanline, ppu_cycle, cpu_cycle);
        // let clock_str = if is_trace_cycles {
        //     format!(" PPU:{:>3},{:>3} CYC:{}", cur_scanline, ppu_cycle, cpu_cycle)
        // } else {
        //     format!("")
        // };

        let trace = format!(
            "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}{}",
            asm_str, reg_a, reg_x, reg_y, status, stack_pointer, clock_str
        )
        .to_ascii_uppercase();

        log.push(trace);
        Ok(())
    }
}

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
    let mut nes = TestNes::new().setup();
    println!("Loading from path");
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
    let mut nes = TestNes::new().setup();
    println!("Loading from path");
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

    let expected_log: Vec<String> = read_to_string("logs/nestest_ppu_cyc.log")
        .expect("Failed to read input")
        .split("\n")
        .map(|s| s.trim_end().to_string())
        .collect();

    for i in 0..5002 {
        let trace_line = nes.program_trace.get(i).expect("Line not found");
        assert_eq!(trace_line, &expected_log[i], "Diff at line {}", i);
    }

    // assert_eq!(cpu.read_byte(0x600), 0);
}
