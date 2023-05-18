use std::collections::HashMap;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::fs::remove_file;
use std::io::Write;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;

use crate::cartridge::ROM;
use crate::controller::Controller;
use crate::controller::ControllerState;
use crate::cpu::CPU;
use crate::cpu::bus::Bus;
use crate::ppu::PPU;
use crate::trace::trace_cpu;

use self::frame::Frame;

pub mod frame;
pub mod palette;
pub mod tile_viewer;

pub fn run(path: &str) {
    // Initialize sdl display
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("NES", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();
    // Key mapping
    let mut key_map = HashMap::new();
    key_map.insert(Keycode::A, ControllerState::A);
    key_map.insert(Keycode::S, ControllerState::B);
    key_map.insert(Keycode::Space, ControllerState::SELECT);
    key_map.insert(Keycode::Return, ControllerState::START);
    key_map.insert(Keycode::Up, ControllerState::UP);
    key_map.insert(Keycode::Down, ControllerState::DOWN);
    key_map.insert(Keycode::Left, ControllerState::LEFT);
    key_map.insert(Keycode::Right, ControllerState::RIGHT);
    // Create a frame
    let mut frame = Frame::new();

    let cartridge = ROM::create_from_nes(path).unwrap();
    let bus = Bus::new(cartridge, move |ppu: &PPU, controller: &mut Controller| {
        frame.render(ppu);
        texture.update(None, frame.as_bytes_ref(), 256 * 3);
        canvas.copy(&texture, None, None);
        canvas.present();
        // let mut controller_state = ControllerState::from_bits_retain(0);
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        controller.controller_state.set(*key, true);
                        // controller_state.insert(*key);
                    }
                },
                Event::KeyUp{ keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        controller.controller_state.set(*key, false);
                        // controller_state.remove(*key);
                    }
                },
                _ => {}
            }
        }
        // controller.set_controller_state(controller_state);
    });
    let mut cpu = CPU::new(bus);
    cpu.reset();

    remove_file("logs/run.log").err();
    let mut f = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("logs/run.log")
        .unwrap();

    // cpu.run_with_callback(move |cpu| {
    //     // if let Ok(s) = trace_cpu(cpu, true) {
    //     //     writeln!(f, "{}", s).expect("Couldn't write line");
    //     // }
    //     let byte = cpu.program_counter;
    //     writeln!(f, "{:x}", byte).expect("Couldn't write line");
    // });
    cpu.run();
}