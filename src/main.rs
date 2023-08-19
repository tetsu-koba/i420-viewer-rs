use nix::poll::{poll, PollFd, PollFlags};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::video::FullscreenType;
use std::io::{ErrorKind, Read};
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
        .hidden()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;
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

    let mut shown = false;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => canvas.window_mut().set_fullscreen(FullscreenType::Off)?,
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => canvas.window_mut().set_fullscreen(FullscreenType::True)?,
                _ => {}
            }
        }
        match poll(&mut [poll_fd], timeout) {
            Ok(0) => continue,
            Ok(_) => {}
            Err(e) if e == nix::errno::Errno::EINTR => continue,
            Err(e) => return Err(Box::new(e)),
        }
        match reader.read_exact(&mut inbuf) {
            Ok(_) => {}
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Box::new(e)),
        }
        texture.update(None, &inbuf, w)?;

        canvas.clear();
        let r2 = calc_dest_rect(canvas.viewport(), width, height);
        canvas.copy(&texture, None, r2)?;
        canvas.present();
        if !shown {
            canvas.window_mut().show();
            shown = true;
        }
    }

    Ok(())
}

// calc inner rect which keeps aspect ratio
fn calc_dest_rect(dst: Rect, width: u32, height: u32) -> Rect {
    let sw = dst.width();
    let sh = dst.height();
    if sw == width && sh == height {
        return dst;
    }
    if sw * height < sh * width {
        let h2 = height * sw / width;
        let y = (sh - h2) / 2;
        Rect::new(0, y as _, sw, h2)
    } else {
        let w2 = width * sh / height;
        let x = (sw - w2) / 2;
        Rect::new(x as _, 0, w2, sh)
    }
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
