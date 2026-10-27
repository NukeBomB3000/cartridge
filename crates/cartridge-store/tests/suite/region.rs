use cartridge_store::Region;

#[test]
fn a_file_name_says_where_its_game_comes_from() {
    assert_eq!(Region::from_tags("Super Metroid (USA)"), Some(Region::Usa));
    assert_eq!(
        Region::from_tags("Super Metroid (Japan, USA) (En,Ja)"),
        Some(Region::Japan)
    );
    assert_eq!(
        Region::from_tags("Super Mario World (Europe) [!]"),
        Some(Region::Europe)
    );
    assert_eq!(Region::from_tags("Chrono Trigger (Rev 1)"), None);
    assert_eq!(Region::from_tags("Untagged"), None);
}
