use std::{os::raw::{c_int, c_void}, collections::HashMap};

use sdl2::{pixels::PixelFormatEnum, keyboard::Keycode, event::Event};

use crate::{controller::ControllerState, nes::{ActionNES, NES}};

use super::frame::Frame;

#[allow(non_camel_case_types)]
type em_callback_func = unsafe extern "C" fn(context: *mut c_void);

extern "C" {
    pub fn emscripten_set_main_loop_arg(
        func: em_callback_func,
        arg: *mut c_void,
        fps: c_int,
        simulate_infinite_loop: c_int,
    );
}

fn setup_mainloop<F: FnMut() + 'static>(
    fps: c_int,
    simulate_infinite_loop: c_int,
    callback: F,
) {
    let on_the_heap = Box::new(callback);
    let leaked_pointer = Box::into_raw(on_the_heap);
    let untyped_pointer = leaked_pointer as *mut c_void;

    unsafe {
        emscripten_set_main_loop_arg(wrapper::<F>, untyped_pointer, fps, simulate_infinite_loop)
    }

    extern "C" fn wrapper<F: FnMut() + 'static>(untyped_pointer: *mut c_void) {
        let leaked_pointer = untyped_pointer as *mut F;
        let callback_ref = unsafe { &mut *leaked_pointer };
        callback_ref()
    }
}

pub fn run_emscripten(path: &str) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("NES", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    // let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();
    let creator = canvas.texture_creator();

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

    let fps = -1; // call the function as fast as the browser wants to render (typically 60fps)
    let simulate_infinite_loop = 1; // call the function repeatedly

    let mut frame = Frame::new();
    let mut nes = ActionNES::new();
    nes.load_from_path(path).unwrap();
    nes.reset().unwrap();

    setup_mainloop(fps, simulate_infinite_loop, move || {
        // 1. Execute until next frame
        nes.next_ppu_frame();

        // 2. Update the display
        
        let mut texture = creator
            .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
            .unwrap();
        frame.render(&nes.ppu_state, &nes.rom);
        texture.update(None, frame.as_bytes_ref(), 256 * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        // 3. Read user input
        // for event in event_pump.poll_iter() {
        //     match event {
        //         Event::Quit { .. }
        //         | Event::KeyDown {
        //             keycode: Some(Keycode::Escape),
        //             ..
        //         } => std::process::exit(0),
        //         Event::KeyDown { keycode, .. } => {
        //             if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
        //                 nes.update_controller(*key, true);
        //                 // controller_state.insert(*key);
        //             }
        //         }
        //         Event::KeyUp { keycode, .. } => {
        //             if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
        //                 nes.update_controller(*key, false);
        //                 // controller_state.remove(*key);
        //             }
        //         }
        //         _ => {}
        //     }
        // }
    })

}