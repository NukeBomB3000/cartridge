//! A game's name as the shelf prints it under its cart: its file's name without the tags in
//! round and square brackets, underscores as spaces, and a name filed as "Legend of Zelda, The"
//! put back the way it is said.

/// How many letters the shelf has room for on one line, at 640 pixels across.
pub const TITLE_MAX: usize = 48;

pub fn clean_title(stem: &str) -> String {
    let mut out = String::new();
    let mut depth = 0u32;
    for c in stem.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            '_' if depth == 0 => out.push(' '),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    let words: Vec<&str> = out.split_whitespace().collect();
    let mut name = words.join(" ");
    // "Legend of Zelda, The - A Link to the Past" and "Legend of Zelda, The"
    for article in ["The", "A", "An", "Die", "Der", "Das", "Le", "La", "Les"] {
        let suffix = format!(", {article}");
        if let Some(at) = name.find(&suffix) {
            let end = at + suffix.len();
            let rest = &name[end..];
            if rest.is_empty() || rest.starts_with(' ') {
                name = format!("{article} {}{}", &name[..at], rest);
                break;
            }
        }
    }
    let name = name.trim().to_string();
    if name.chars().count() > TITLE_MAX {
        let cut: String = name.chars().take(TITLE_MAX - 3).collect();
        format!("{}...", cut.trim_end())
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tags_go_and_the_name_stays() {
        assert_eq!(clean_title("Advance Wars (USA)"), "Advance Wars");
        assert_eq!(
            clean_title("Pokemon - Red Version (USA, Europe) (SGB Enhanced) [!]"),
            "Pokemon - Red Version"
        );
        assert_eq!(clean_title("Super_Mario_Land"), "Super Mario Land");
        assert_eq!(clean_title("  Tetris  (World) (Rev 1) "), "Tetris");
    }

    #[test]
    fn a_name_filed_by_its_article_is_said_with_it_first() {
        assert_eq!(
            clean_title("Legend of Zelda, The (USA)"),
            "The Legend of Zelda"
        );
        assert_eq!(
            clean_title("Legend of Zelda, The - A Link to the Past (USA)"),
            "The Legend of Zelda - A Link to the Past"
        );
        assert_eq!(clean_title("Lion King, The"), "The Lion King");
        assert_eq!(clean_title("Theme Park"), "Theme Park");
    }

    #[test]
    fn a_long_name_is_cut_to_the_line() {
        let long = "A".repeat(80);
        let t = clean_title(&long);
        assert_eq!(t.chars().count(), TITLE_MAX);
        assert!(t.ends_with("..."));
    }
}
