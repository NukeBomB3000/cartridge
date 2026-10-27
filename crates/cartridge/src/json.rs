//! Just enough JSON to read a ScreenScraper reply. It forgives what that server is known to
//! send: a backslash before a character JSON has no escape for is kept as written, rather
//! than sinking the whole reply over one synopsis.

#[derive(Clone, PartialEq, Debug)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    List(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    pub fn parse(text: &str) -> Option<Json> {
        let mut p = Parser {
            s: text.as_bytes(),
            at: 0,
        };
        let value = p.value()?;
        p.space();
        (p.at == p.s.len()).then_some(value)
    }

    /// The member called `key`, if this is an object that has one.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(members) => members.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn list(&self) -> &[Json] {
        match self {
            Json::List(items) => items,
            _ => &[],
        }
    }

    pub fn text(&self) -> Option<&str> {
        match self {
            Json::Text(s) => Some(s),
            _ => None,
        }
    }

    /// A count, whether the server wrote it as a number or, as ScreenScraper does, a string.
    pub fn count(&self) -> Option<u64> {
        match self {
            Json::Number(n) if *n >= 0.0 => Some(*n as u64),
            Json::Text(s) => s.trim().parse().ok(),
            _ => None,
        }
    }
}

struct Parser<'a> {
    s: &'a [u8],
    at: usize,
}

impl Parser<'_> {
    fn space(&mut self) {
        while self.s.get(self.at).is_some_and(u8::is_ascii_whitespace) {
            self.at += 1;
        }
    }

    fn eat(&mut self, word: &[u8]) -> bool {
        let found = self.s[self.at..].starts_with(word);
        if found {
            self.at += word.len();
        }
        found
    }

    fn value(&mut self) -> Option<Json> {
        self.space();
        match *self.s.get(self.at)? {
            b'{' => self.object(),
            b'[' => self.list(),
            b'"' => self.string().map(Json::Text),
            b't' => self.eat(b"true").then_some(Json::Bool(true)),
            b'f' => self.eat(b"false").then_some(Json::Bool(false)),
            b'n' => self.eat(b"null").then_some(Json::Null),
            _ => self.number(),
        }
    }

    fn object(&mut self) -> Option<Json> {
        self.at += 1;
        let mut members = Vec::new();
        self.space();
        if self.eat(b"}") {
            return Some(Json::Object(members));
        }
        loop {
            self.space();
            let key = self.string()?;
            self.space();
            if !self.eat(b":") {
                return None;
            }
            members.push((key, self.value()?));
            self.space();
            if self.eat(b"}") {
                return Some(Json::Object(members));
            }
            if !self.eat(b",") {
                return None;
            }
        }
    }

    fn list(&mut self) -> Option<Json> {
        self.at += 1;
        let mut items = Vec::new();
        self.space();
        if self.eat(b"]") {
            return Some(Json::List(items));
        }
        loop {
            items.push(self.value()?);
            self.space();
            if self.eat(b"]") {
                return Some(Json::List(items));
            }
            if !self.eat(b",") {
                return None;
            }
        }
    }

    fn number(&mut self) -> Option<Json> {
        let start = self.at;
        while self
            .s
            .get(self.at)
            .is_some_and(|c| c.is_ascii_digit() || b"+-.eE".contains(c))
        {
            self.at += 1;
        }
        let text = std::str::from_utf8(&self.s[start..self.at]).ok()?;
        text.parse().ok().map(Json::Number)
    }

    fn hex4(&mut self) -> Option<u32> {
        let digits = self.s.get(self.at..self.at + 4)?;
        let n = u32::from_str_radix(std::str::from_utf8(digits).ok()?, 16).ok()?;
        self.at += 4;
        Some(n)
    }

    fn string(&mut self) -> Option<String> {
        if !self.eat(b"\"") {
            return None;
        }
        let mut out: Vec<u8> = Vec::new();
        loop {
            let c = *self.s.get(self.at)?;
            self.at += 1;
            match c {
                b'"' => return String::from_utf8(out).ok(),
                b'\\' => {
                    let e = *self.s.get(self.at)?;
                    self.at += 1;
                    let plain = match e {
                        b'"' => Some(b'"'),
                        b'\\' => Some(b'\\'),
                        b'/' => Some(b'/'),
                        b'b' => Some(8),
                        b'f' => Some(12),
                        b'n' => Some(b'\n'),
                        b'r' => Some(b'\r'),
                        b't' => Some(b'\t'),
                        _ => None,
                    };
                    if let Some(p) = plain {
                        out.push(p);
                    } else if e == b'u' {
                        let ch = self.unicode()?;
                        let mut buf = [0; 4];
                        out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                    } else {
                        // Not an escape JSON knows: the server meant the backslash itself.
                        self.at -= 1;
                        out.push(b'\\');
                    }
                }
                _ => out.push(c),
            }
        }
    }

    fn unicode(&mut self) -> Option<char> {
        let first = self.hex4()?;
        if (0xd800..0xdc00).contains(&first) && self.eat(b"\\u") {
            let low = self.hex4()?;
            let code = 0x10000 + ((first - 0xd800) << 10) + (low.wrapping_sub(0xdc00) & 0x3ff);
            return Some(char::from_u32(code).unwrap_or('\u{fffd}'));
        }
        Some(char::from_u32(first).unwrap_or('\u{fffd}'))
    }
}
