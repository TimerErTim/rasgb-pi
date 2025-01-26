use crate::display::Display;
use crate::frame::filler::FrameFiller;
use crate::frame::gen::FrameGenerator;
use std::time::SystemTime;

pub fn sync_frames(
    display: &impl Display,
    filler: &impl FrameFiller,
    frames: &impl FrameGenerator,
) {
    let current_time = SystemTime::now();
    let timestamp = current_time
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("time went backwards");
    let unix_micros = timestamp.as_micros();
    loop {
        let frame = frames.generate(unix_micros);
        if let Some(frame) = frame {
            if let Err(e) = filler.push_to_display(frame, display) {
                eprintln!("failed to push frame: {}", e);
                continue;
            }
        }
        break;
    }
}
