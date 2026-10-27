use cartridge::core_options::{
    kept_options, shown, CoreOptionsScreen, Outcome, Page, CORE_OPTIONS_FILE,
};
use cartridge_input::{Action, Btn};
use cartridge_retro::CoreOption;
use cartridge_store::Core;

fn option(key: &str, values: &[&str], value: &str) -> CoreOption {
    CoreOption {
        key: key.into(),
        label: key.into(),
        values: values.iter().map(|v| v.to_string()).collect(),
        value: value.into(),
    }
}

fn screen() -> CoreOptionsScreen {
    CoreOptionsScreen {
        title: "DSPERATE".into(),
        page: Page::Options,
        row: 0,
        options: vec![
            option(
                "dsperate_layout",
                &["Top Dominant", "Vertical", "Picture in Picture"],
                "Top Dominant",
            ),
            option("dsperate_frameskip", &["Auto", "Off", "1"], "Auto"),
        ],
        at: 0,
    }
}

#[test]
fn left_and_right_step_the_lit_option_round_its_values_and_say_so() {
    let mut s = screen();
    let press = |s: &mut CoreOptionsScreen, b| s.input(Action::GbaDown(b));
    assert_eq!(
        press(&mut s, Btn::Right),
        Outcome::Set("dsperate_layout".into(), "Vertical".into())
    );
    assert_eq!(
        press(&mut s, Btn::Left),
        Outcome::Set("dsperate_layout".into(), "Top Dominant".into())
    );
    assert_eq!(
        press(&mut s, Btn::Left),
        Outcome::Set("dsperate_layout".into(), "Picture in Picture".into()),
        "did not go round from the first value"
    );
    press(&mut s, Btn::Down);
    assert_eq!(
        press(&mut s, Btn::Right),
        Outcome::Set("dsperate_frameskip".into(), "Off".into())
    );
    press(&mut s, Btn::Down);
    assert_eq!(s.at, 0, "down from the last row did not wrap");
    assert_eq!(
        press(&mut s, Btn::B),
        Outcome::Stay,
        "B left the menu from its options"
    );
    assert_eq!(s.page, Page::Pause);
    assert_eq!(press(&mut s, Btn::B), Outcome::Close);
}

#[test]
fn only_a_cores_own_kept_options_come_back_for_it() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("Config")).unwrap();
    std::fs::write(
        d.path().join(CORE_OPTIONS_FILE),
        "dsperate_layout = Vertical\nswanstation_GPU_Renderer = Software\n",
    )
    .unwrap();
    assert_eq!(
        kept_options(d.path(), Core::Dsperate),
        [("dsperate_layout".to_string(), "Vertical".to_string())]
    );
    assert!(kept_options(d.path(), Core::Mgba).is_empty());
}

#[test]
fn the_pause_menu_saves_loads_opens_the_states_and_its_options_and_quits() {
    let mut s = CoreOptionsScreen::default();
    let press = |s: &mut CoreOptionsScreen, b| s.input(Action::GbaDown(b));
    assert_eq!(press(&mut s, Btn::A), Outcome::SaveState);
    press(&mut s, Btn::Down);
    assert_eq!(press(&mut s, Btn::A), Outcome::LoadState);
    press(&mut s, Btn::Down);
    assert_eq!(press(&mut s, Btn::A), Outcome::States);
    press(&mut s, Btn::Down);
    assert_eq!(press(&mut s, Btn::A), Outcome::Stay);
    assert_eq!(s.page, Page::Options);
    press(&mut s, Btn::B);
    press(&mut s, Btn::Down);
    assert_eq!(
        press(&mut s, Btn::A),
        Outcome::Close,
        "RESUME did not resume"
    );
    press(&mut s, Btn::Down);
    assert_eq!(press(&mut s, Btn::A), Outcome::Quit);
    assert_eq!(s.input(Action::GameMenu), Outcome::Close);
}

#[test]
fn dsperates_menu_keeps_only_the_options_a_player_has_a_use_for() {
    let keep: Vec<&str> = [
        "dsperate_layout",
        "dsperate_screen_gap",
        "dsperate_swap_button",
        "dsperate_stylus_control",
        "dsperate_cpu_engine",
        "dsperate_frameskip",
    ]
    .into_iter()
    .filter(|k| shown(Core::Dsperate, &option(k, &["a", "b"], "a")))
    .collect();
    assert_eq!(
        keep,
        [
            "dsperate_layout",
            "dsperate_screen_gap",
            "dsperate_frameskip"
        ]
    );
    assert!(shown(
        Core::Swanstation,
        &option("swanstation_GPU_Renderer", &["a"], "a")
    ));
}
