use nix::poll::{poll, PollFd, PollFlags};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::io::Read;
use std::os::fd::AsRawFd;

pub fn i420_viewer(
    reader: &mut std::fs::File,
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

    let w = width as usize;
    let h = height as usize;
    let mut inbuf = vec![0; w * h * 3 / 2];

    let poll_fd = PollFd::new(reader.as_raw_fd(), PollFlags::POLLIN);
    let timeout = 1000; // msec

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
        if poll(&mut [poll_fd], timeout)? != 0 {
            reader.read_exact(&mut inbuf)?;
            texture.update(None, &inbuf, w)?;

            canvas.clear();
            canvas.copy(&texture, None, None)?;
            canvas.present();
        }
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
