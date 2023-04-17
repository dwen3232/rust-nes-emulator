use std::ops::Add;

use crate::{
    cpu::{
        CPU, Param,
        decode::{
            Instruction, AddressingMode, decode_opcode
        },
        bus::Bus
    }, 
    cartridge::test::test_rom,
    traits::Memory
};

pub fn trace_cpu(cpu: &mut CPU) -> Result<String, String> {
    let prev_counter = cpu.program_counter;

    // SOME TEMP STUFF FOR DEBUGGING:
    // if prev_counter == 0xCFDB {
    //     println!("Found {:04x} at 0x0080", cpu.read_byte(0x0080));
    // }
    // if prev_counter == 0xCFF2 {
    //     println!("Found {:04x} at 0x00FF", cpu.read_byte(0x00FF));
    //     println!("Found {:04x} at 0x0000", cpu.read_byte(0x0000));
    //     println!("Found {:04x} at 0x0400", cpu.read_byte(0x0400));
    //     println!("Found {:04x} at 0x0080", cpu.read_byte(0x0080));
    // }

    // decode instruction and addressing mode
    let opcode = cpu.read_byte_from_pc();
    let (instruction, addressing_mode) = decode_opcode(opcode)?;

    // parse instruction parameter 
    let param = cpu.read_arg(&addressing_mode);
    let length = cpu.program_counter - prev_counter;

    cpu.program_counter = prev_counter;     // revert program_counter

    let mut hex_dump = vec![];
    // add opcode byte to dump
    hex_dump.push(opcode);


    // get the parsed arg as a u16
    let arg = match length {
        1 => 0,
        2 => {
            let address: u8 = cpu.read_byte(prev_counter + 1);
            hex_dump.push(address);
            address as u16
        },
        3 => {
            
            let address_lo = cpu.read_byte(prev_counter + 1);
            let address_hi = cpu.read_byte(prev_counter + 2);
            hex_dump.push(address_lo);
            hex_dump.push(address_hi);

            let address = cpu.read_two_bytes(prev_counter + 1);
            address
        },
        _ => {panic!()}
    };

    // create temp string for operand details
    let tmp = match (&instruction, addressing_mode, param) {
        // length 1
        (_, AddressingMode::Implicit, _) => String::from(""),
        (_, AddressingMode::Accumulator, _) => {
            format!("A")
        },
        // length 2
        (_, AddressingMode::Immediate, Some(Param::Value(value))) => {
            format!("#${:02x}", value)
        },
        (_, AddressingMode::ZeroPage, Some(Param::Address(address))) => {
            let stored_value = cpu.read_byte(address);
            format!("${:02x} = {:02x}", address, stored_value)
        },
        (_, AddressingMode::ZeroPageIndexX, Some(Param::Address(address))) => {
            let stored_value = cpu.read_byte(address);
            format!(
                "${:02x},X @ {:02x} = {:02x}",
                arg, address, stored_value
            )
        },
        (_, AddressingMode::ZeroPageIndexY, Some(Param::Address(address))) => {
            let stored_value = cpu.read_byte(address);
            format!(
                "${:02x},Y @ {:02x} = {:02x}",
                arg, address, stored_value
            )
        },
        (_, AddressingMode::IndirectX, Some(Param::Address(address))) => {
            let stored_value = cpu.read_byte(address);
            format!(
                "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                arg,
                (arg.wrapping_add(cpu.reg_x as u16) as u8),
                address,
                stored_value
            )
        },
        (_, AddressingMode::IndirectY, Some(Param::Address(address))) => {
            let stored_value = cpu.read_byte(address);
            format!(
                "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                arg,
                (address.wrapping_sub(cpu.reg_y as u16)),
                address,
                stored_value
            )
        },
        (_, AddressingMode::Relative, _) => {
            let address: usize =
            (prev_counter as usize + 2).wrapping_add((arg as i8) as usize);
            format!("${:04x}", address)
        },
        // length 3
        (_, AddressingMode::IndirectJump, Some(Param::Address(address))) => {
            format!("(${:04x}) = {:04x}", arg, address)
        },
        (_, AddressingMode::AbsoluteJump, Some(Param::Address(address))) => {
            format!("${:04x}", address)
        },
        (_, AddressingMode::Absolute, Some(Param::Address(address))) => {
            let stored_value = cpu.read_byte(address);
            format!("${:04x} = {:02x}", address, stored_value)
        },
        (_, AddressingMode::AbsoluteIndexX, Some(Param::Address(address))) => {
            let stored_value = cpu.read_byte(address);
            format!(
                "${:04x},X @ {:04x} = {:02x}",
                arg, address, stored_value
            )
        },
        (_, AddressingMode::AbsoluteIndexY, Some(Param::Address(address))) => {
            let stored_value = cpu.read_byte(address);
            format!(
                "${:04x},Y @ {:04x} = {:02x}",
                arg, address, stored_value
            )
        },
        (instruction, mode, param) => {
            panic!("Could not trace this argument {:?}, {:?}, {:?}", instruction, mode, param)
        }    
    };

    // add strings together
    let opstring = format!("{:?}", instruction);
    let hex_str = hex_dump
        .iter()
        .map(|z| format!("{:02x}", z))
        .collect::<Vec<String>>()
        .join(" ");
    let asm_str = format!("{:04x}  {:8} {: >4} {}", prev_counter, hex_str, opstring, tmp)
        .trim()
        .to_string();

    let trace = format!(
        "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}",
        asm_str, cpu.reg_a, cpu.reg_x, cpu.reg_y, cpu.status, cpu.stack_pointer,
    )
    .to_ascii_uppercase();

    Ok(trace)
}


#[test]
fn test_format_trace_cpu() {
    let mut bus = Bus::new(test_rom());
    bus.write_byte(100, 0xa2);
    bus.write_byte(101, 0x01);
    bus.write_byte(102, 0xca);
    bus.write_byte(103, 0x88);
    bus.write_byte(104, 0x00);

    let mut cpu = CPU::new(bus);
    cpu.program_counter = 0x64;
    cpu.reg_a = 1;
    cpu.reg_x = 2;
    cpu.reg_y = 3;
    let mut result: Vec<String> = vec![];
    cpu.run_with_callback(|cpu| {
        if let Ok(s) = trace_cpu(cpu) {
            result.push(s);
        }
    });
    assert_eq!(
        "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
        result[0]
    );
    assert_eq!(
        "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
        result[1]
    );
    assert_eq!(
        "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
        result[2]
    );
}

#[test]
fn test_format_mem_access() {
    let mut bus = Bus::new(test_rom());
    // ORA ($33), Y
    bus.write_byte(100, 0x11);
    bus.write_byte(101, 0x33);


    //data
    bus.write_byte(0x33, 00);
    bus.write_byte(0x34, 04);

    //target cell
    bus.write_byte(0x400, 0xAA);

    let mut cpu = CPU::new(bus);
    cpu.program_counter = 0x64;
    cpu.reg_y = 0;
    let mut result: Vec<String> = vec![];
    cpu.run_with_callback(|cpu| {
        if let Ok(s) = trace_cpu(cpu) {
            result.push(s);
        }
    });
    assert_eq!(
        "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
        result[0]
    );
}
