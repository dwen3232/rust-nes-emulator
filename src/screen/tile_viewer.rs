// use std::mem::transmute;

// use sdl2::event::Event;
// use sdl2::keyboard::Keycode;
// use sdl2::pixels::Color;
// use sdl2::pixels::PixelFormatEnum;
// use sdl2::EventPump;

// use crate::cartridge::ROM;

// use super::frame;
// use super::frame::Frame;
// use super::palette;


// fn show_tile(chr_rom: &Vec<u8>, bank: usize, tile_n: usize) -> Frame {
//     let mut frame = Frame::new();
//     let bank = (bank * 0x1000) as usize;

//     let tile_range = (bank + 16 * tile_n)..(bank + 16 * (tile_n + 1));
//     debug_assert_eq!(tile_range.len(), 16);
//     println!("{:?}", tile_range);

//     let tile = &chr_rom[tile_range];
//     debug_assert_eq!(tile.len(), 16);
//     println!("{:?}", tile);

//     let (upper, lower) = tile.split_at(8);

//     for y in 0..8 {
//         let mut hi = upper[y];
//         let mut lo = lower[y];

//         for x in (0..8).rev() {
//             let hi_bit = (hi & 1) == 1;
//             let lo_bit = (lo & 1) == 1;
//             hi = hi >> 1;
//             lo = lo >> 1;

//             let rgb = match (hi_bit, lo_bit) {
//                 (false, false) => palette::SYSTEM_PALLETE[0x01],
//                 (false, true) => palette::SYSTEM_PALLETE[0x23],
//                 (true, false) => palette::SYSTEM_PALLETE[0x27],
//                 (true, true) => palette::SYSTEM_PALLETE[0x30],
//             };
//             frame.set_pixel(x, y, rgb);
//         }
//     }
//     frame

// }

// pub fn run() {
//     let sdl_context = sdl2::init().unwrap();
//     let video_subsystem = sdl_context.video().unwrap();
//     let window = video_subsystem
//         .window("Tile viewer", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
//         .position_centered()
//         .build()
//         .unwrap();

//     let mut canvas = window.into_canvas().present_vsync().build().unwrap();
//     let mut event_pump = sdl_context.event_pump().unwrap();
//     canvas.set_scale(3.0, 3.0).unwrap();

//     let creator = canvas.texture_creator();
//     let mut texture = creator
//         .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
//         .unwrap();
//     // load the game
//     let rom = ROM::create_from_nes("game_roms/pacman.nes").unwrap();


//     let tile_frame = show_tile(&rom.chr_rom, 1,0);

//     // let's do some alchemy
//     let pixel_data: &[u8; frame::WIDTH * frame::HEIGHT * 3] = unsafe { transmute(&tile_frame.data) };

//     texture.update(None, pixel_data, 256 * 3).unwrap();
//     canvas.copy(&texture, None, None).unwrap();
//     canvas.present();

//     loop {
//         for event in event_pump.poll_iter() {
//             match event {
//             Event::Quit { .. }
//             | Event::KeyDown {
//                 keycode: Some(Keycode::Escape),
//                 ..
//             } => std::process::exit(0),
//             _ => { /* do nothing */ }
//             }
//         }
//     }

// }

