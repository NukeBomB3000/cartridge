#[test]
fn tags_are_the_groups_clean_label_drops() {
    let stem = "Pokemon - LeafGreen Version (USA, Europe) (Rev 1)";
    assert_eq!(
        cartridge_ui::label_tags(stem),
        vec!["USA, Europe".to_string(), "Rev 1".to_string()],
        "one tag per bracketed group, in filename order"
    );
    assert_eq!(cartridge_ui::clean_label(stem), "Pokemon LeafGreen Version");
}

#[test]
fn a_cart_with_no_tags_gets_none() {
    assert!(cartridge_ui::label_tags("Metroid Fusion").is_empty());
}

#[test]
fn a_comma_inside_one_group_does_not_make_two_tags() {
    assert_eq!(cartridge_ui::label_tags("Game (USA, Europe)").len(), 1);
}
