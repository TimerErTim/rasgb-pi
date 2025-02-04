use super::*;



#[test]
fn test_obsolete_with_higher_channel_always_false() {
    let gen = ChannelTimeQueuedFrameGenerator::new(2500, 1.0);
    gen.add_frame(0, 100, Frame::empty());

    assert!(!gen.is_frame_superseded(1, 100));
    assert!(!gen.is_frame_superseded(1, 150));
    assert!(!gen.is_frame_superseded(1, 50));
}


#[test]
fn test_obsolete_with_lower_channel_true_when_later() {
    let gen = ChannelTimeQueuedFrameGenerator::new(2500, 1.0);
    gen.add_frame(1, 100, Frame::empty());

    
    assert!(gen.is_frame_superseded(0, 100));
    assert!(gen.is_frame_superseded(0, 200));
}

#[test]
fn test_obsolete_with_lower_channel_false_when_outside_range() {
    let gen = ChannelTimeQueuedFrameGenerator::new(2500, 1.0);
    gen.add_frame(1, 100, Frame::empty());

    assert!(!gen.is_frame_superseded(0, 50));
    assert!(!gen.is_frame_superseded(0, 1_000_000 + 1_000));
}

#[test]
fn test_obsolete_with_same_channel_false_when_inside_range() {
    let gen = ChannelTimeQueuedFrameGenerator::new(2500, 1.0);
    gen.add_frame(1, 100, Frame::empty());

    assert!(!gen.is_frame_superseded(1, 50));
    assert!(!gen.is_frame_superseded(1, 150));
}
