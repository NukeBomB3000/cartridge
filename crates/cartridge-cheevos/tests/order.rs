use cartridge_cheevos::{order, Achievement};

fn a(title: &str, at: i64) -> Achievement {
    Achievement {
        title: title.into(),
        description: String::new(),
        points: 5,
        unlocked: at > 0,
        unlocked_at: at,
        badge_url: String::new(),
    }
}

#[test]
fn the_earned_come_first_newest_first_and_the_rest_keep_their_places() {
    let mut list = vec![
        a("locked one", 0),
        a("old", 100),
        a("locked two", 0),
        a("new", 300),
        a("middle", 200),
    ];
    order(&mut list);
    let titles: Vec<&str> = list.iter().map(|x| x.title.as_str()).collect();
    assert_eq!(titles, ["new", "middle", "old", "locked one", "locked two"]);
}
