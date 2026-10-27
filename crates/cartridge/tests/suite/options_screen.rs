use cartridge::core_options::{kept_options, CoreOptionsScreen, Outcome, CORE_OPTIONS_FILE};
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
fn l2_steps_the_ds_layouts_round() {
    use cartridge::core_options::{next_ds_layout, DS_LAYOUTS};
    let mut at = DS_LAYOUTS[0];
    for want in DS_LAYOUTS.iter().skip(1).chain(DS_LAYOUTS.iter().take(1)) {
        at = next_ds_layout(at);
        assert_eq!(at, *want);
    }
    assert_eq!(next_ds_layout("something else"), DS_LAYOUTS[1]);
}
