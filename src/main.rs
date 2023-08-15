use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::io::Read;

pub fn i420_viewer(
    reader: &mut dyn Read,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

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
        let w = width as usize;
        let h = height as usize;
        let mut inbuf = vec![0; w * h * 3 / 2];
        reader.read_exact(&mut inbuf)?;

        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..h {
                for x in 0..w {
                    let offset = y * pitch + x;
                    buffer[offset] = inbuf[y * w + x];
                }
            }
            let y_size = pitch * h;
            for y in 0..h / 2 {
                for x in 0..w / 2 {
                    let u_offset = y_size + y * pitch / 2 + x;
                    let v_offset = y_size + (pitch / 2 * h / 2) + y * pitch / 2 + x;
                    buffer[u_offset] = inbuf[w * h + y * w / 2 + x];
                    buffer[v_offset] = inbuf[w * h + (w / 2 * h / 2) + y * w / 2 + x];
                }
            }
        })?;

        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();
    }

    Ok(())
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} input_file width height", args[0]);
        std::process::exit(1);
    }
    let width: u32 = args[2].parse()?;
    let height: u32 = args[3].parse()?;
    let mut reader = std::fs::File::open(&args[1])?;
    i420_viewer(&mut reader, width, height)?;

    Ok(())
}
