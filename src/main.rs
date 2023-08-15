use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::time::Duration;
//use sdl2::rect::Rect;

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let width = 256;
    let height = 256;
    let window = video_subsystem
        .window("i420-viewer-rs", width, height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::IYUV, width, height)
        .map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    let mut count = 0;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        // Create a U-V gradient
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            // `pitch` is the width of the Y component
            // The U and V components are half the width and height of Y

            let w = width as usize;
            let h = height as usize;

            // Set Y (constant)
            for y in 0..h {
                for x in 0..w {
                    let offset = y * pitch + x;
                    buffer[offset] = 128;
                }
            }

            let y_size = pitch * h;

            // Set U and V (X and Y)
            for y in 0..h / 2 {
                for x in 0..w / 2 {
                    let u_offset = y_size + y * pitch / 2 + x;
                    let v_offset = y_size + (pitch / 2 * h / 2) + y * pitch / 2 + x;
                    buffer[u_offset] = ((x * 2) + count) as _;
                    buffer[v_offset] = ((y * 2) + count) as _;
                }
            }
        })?;

        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();

        count += 1;
        std::thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}
