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

use crate::nes::ActionNES;
use crate::nes::NES;
use crate::rom::ROM;
use crate::controller::Controller;
use crate::controller::ControllerState;

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
    key_map.insert(Keycode::Q, ControllerState::SELECT);
    key_map.insert(Keycode::W, ControllerState::START);
    key_map.insert(Keycode::Up, ControllerState::UP);
    key_map.insert(Keycode::Down, ControllerState::DOWN);
    key_map.insert(Keycode::Left, ControllerState::LEFT);
    key_map.insert(Keycode::Right, ControllerState::RIGHT);
    // Create a frame
    let mut frame = Frame::new();
    let mut nes = ActionNES::new();
    nes.load_from_path(path);

    loop {
        // 1. Execute until next frame
        nes.next_ppu_frame();
        println!("executing");

        // 2. Update the display
        frame.render(&nes.ppu_state, &nes.rom);
        texture.update(None, frame.as_bytes_ref(), 256 * 3);
        canvas.copy(&texture, None, None);
        canvas.present();
        println!("Updating display");

        // 3. Read user input 
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        nes.update_controller(*key, true);
                        // controller_state.insert(*key);
                    }
                },
                Event::KeyUp{ keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        nes.update_controller(*key, false);
                        // controller_state.remove(*key);
                    }
                },
                _ => {}
            }
        }
    }
}