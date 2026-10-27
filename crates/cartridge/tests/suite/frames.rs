use cartridge::frames::Frames;

fn publish(frames: &Frames, value: u8, size: usize) {
    let mut buf = frames.take_write();
    buf.clear();
    buf.resize(size, value);
    frames.publish(buf, (1, size as u32 / 4));
}

#[test]
fn the_reader_gets_the_newest_frame_and_never_the_same_one_twice() {
    let f = Frames::new(4);
    for i in 0..50u8 {
        publish(&f, i, 4);
    }
    let latest = f.latest().expect("a published frame should be readable");
    assert_eq!(
        &latest[..],
        &[49u8; 4],
        "the reader was handed a stale frame"
    );
    drop(latest);
    assert!(
        f.latest().is_none(),
        "a consumed frame was handed out twice"
    );
}

#[test]
fn buffers_are_recycled_rather_than_reallocated_per_frame() {
    let f = Frames::new(4);
    publish(&f, 1, 4);
    let held = f.latest().expect("a published frame should be readable");
    for i in 0..500u32 {
        publish(&f, i as u8, 4);
    }
    drop(held);
    publish(&f, 0, 4);
    assert!(
        f.allocated() <= 3,
        "{} buffers in flight, the pool is leaking",
        f.allocated()
    );
}

#[test]
fn a_frame_carries_its_own_size_to_the_reader() {
    let f = Frames::new(4);
    for (w, h) in [(256u32, 224u32), (640, 480), (320, 240)] {
        let mut buf = f.take_write();
        buf.clear();
        buf.resize((w * h * 4) as usize, 7);
        f.publish(buf, (w, h));
        let frame = f.latest().expect("a published frame should be readable");
        assert_eq!(frame.size(), (w, h));
        assert_eq!(frame.len(), (w * h * 4) as usize);
    }
}
