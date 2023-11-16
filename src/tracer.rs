use crate::{
    nes::{ActionNES, NES}, 
    cpu::{
        Instruction, InstructionMetaData, 
        CpuState, CpuBus, 
        AddressingMode, Param
    }, 
    ppu::PpuState
};

type ProgramTrace = Vec<String>;
pub struct TraceNes {
    nes: ActionNES,
    pub program_trace: ProgramTrace,
}

impl TraceNes {
    pub fn new() -> Self {
        TraceNes {
            nes: ActionNES::new(),
            program_trace: vec![]
        }
    }

    /// NOTE: this is only used for testing, because the nestest has a unique set up, not sure why
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

        let trace = format!(
            "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}{}",
            asm_str, reg_a, reg_x, reg_y, status, stack_pointer, clock_str
        )
        .to_ascii_uppercase();

        log.push(trace);
        Ok(())
    }
}
