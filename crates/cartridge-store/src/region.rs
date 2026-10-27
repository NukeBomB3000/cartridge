/// Where a game was released, as far as the shelf cares: a SNES game from the USA comes in the
/// American cart, one from anywhere else in the Super Famicom's.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Region {
    Japan,
    Usa,
    Europe,
    Other,
}

impl Region {
    /// The region a file name's tags say, such as `(USA)`, `(Japan)` or `(Europe)`: the first
    /// tag that names one wins, and a tag naming several, such as `(USA, Europe)`, counts as its
    /// first.
    pub fn from_tags(stem: &str) -> Option<Region> {
        tags(stem).iter().find_map(|tag| {
            tag.split(',').map(str::trim).find_map(|word| {
                match word.to_ascii_lowercase().as_str() {
                    "usa" | "us" | "u" => Some(Region::Usa),
                    "japan" | "jp" | "j" => Some(Region::Japan),
                    "europe" | "eu" | "e" | "pal" => Some(Region::Europe),
                    "world" => Some(Region::Usa),
                    "france" | "germany" | "spain" | "italy" | "uk" | "australia" | "sweden"
                    | "netherlands" => Some(Region::Europe),
                    "korea" | "brazil" | "asia" | "china" => Some(Region::Other),
                    _ => None,
                }
            })
        })
    }
}

/// The groups in brackets of a file name, `"Game (USA) [!]"` giving `USA` and `!`.
fn tags(stem: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0u32;
    let mut cur = String::new();
    for ch in stem.chars() {
        match ch {
            '(' | '[' => {
                depth += 1;
                if depth == 1 {
                    cur.clear();
                    continue;
                }
            }
            ')' | ']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let t = cur.trim();
                    if !t.is_empty() {
                        out.push(t.to_string());
                    }
                    continue;
                }
            }
            _ => {}
        }
        if depth >= 1 {
            cur.push(ch);
        }
    }
    out
}
