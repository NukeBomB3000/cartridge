use cartridge_retro::{ButtonMask, MockCore, RetroCore};

#[test]
fn mock_core_round_trips_state() {
    let mut c = MockCore::new();
    c.load(std::path::Path::new("unused")).unwrap();
    for _ in 0..10 {
        c.run_frame(ButtonMask::default());
    }
    let s = c.serialize().unwrap();
    let a = c.video_xrgb8888().to_vec();
    for _ in 0..10 {
        c.run_frame(ButtonMask::default());
    }
    assert_ne!(c.video_xrgb8888(), &a[..]);
    c.unserialize(&s).unwrap();
    assert_eq!(c.video_xrgb8888(), &a[..]);
}

#[test]
fn audio_tracks_the_frame_rate_without_drift() {
    let mut c = MockCore::new();
    c.load(std::path::Path::new("unused")).unwrap();
    let info = c.av_info();
    let frames = 597;
    let mut samples = 0;
    for _ in 0..frames {
        c.run_frame(ButtonMask::default());
        let chunk = c.take_audio();
        assert_eq!(chunk.len() % 2, 0, "audio must be interleaved stereo");
        samples += chunk.len();
    }
    let want = (frames as f64 / info.fps * info.sample_rate) as usize;
    let got = samples / 2;
    assert!(
        got.abs_diff(want) <= 1,
        "{got} stereo frames over {frames} video frames, expected {want}"
    );
}

#[test]
fn a_sized_mock_draws_its_size_and_changes_it_when_told() {
    let mut c = MockCore::sized(320, 240).resized_at(3, 640, 480);
    c.load(std::path::Path::new("unused")).unwrap();
    let mut sizes = vec![c.video_size()];
    for _ in 0..4 {
        c.run_frame(ButtonMask::default());
        sizes.push(c.video_size());
        let (w, h) = c.video_size();
        assert_eq!(c.video_xrgb8888().len(), (w * h * 4) as usize);
    }
    assert_eq!(
        sizes,
        [(320, 240), (320, 240), (320, 240), (640, 480), (640, 480)]
    );
    c.unserialize(&1u64.to_le_bytes()).unwrap();
    assert_eq!(
        c.video_size(),
        (320, 240),
        "a state from before the change kept the new size"
    );
}

#[test]
fn a_pal_mock_runs_at_fifty_frames_a_second() {
    let mut c = MockCore::new().at_fps(50.0);
    c.load(std::path::Path::new("unused")).unwrap();
    assert_eq!(c.av_info().fps, 50.0);
    let mut samples = 0;
    for _ in 0..50 {
        c.run_frame(ButtonMask::default());
        samples += c.take_audio().len() / 2;
    }
    assert_eq!(
        samples,
        c.av_info().sample_rate as usize,
        "a second of sound in fifty frames"
    );
}

#[test]
fn a_mock_on_several_discs_swaps_only_to_one_it_has() {
    let mut c = MockCore::new().with_discs(2);
    assert_eq!((c.disc_count(), c.disc_index()), (2, 0));
    assert!(c.set_disc(1));
    assert_eq!(c.disc_index(), 1);
    assert!(!c.set_disc(2), "a third disc of two went in");
    assert_eq!(c.disc_index(), 1);
    assert_eq!(c.disc_path(0).as_deref(), Some("mock disc 1"));
    assert_eq!(MockCore::new().disc_count(), 1);
}
