//! The game guide's text: hard-wrapped prose joined back into paragraphs, tables and drawings
//! kept as they are, headings found, HTML read for what it says, and a place in the guide
//! that outlives the size it was read at.

use cartridge_ui::guide::{decode, lay_out, parse, parse_html, parse_txt, Advances, Block, Word};

fn text(words: &[Word]) -> String {
    words
        .iter()
        .map(|w| w.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn paras(blocks: &[Block]) -> Vec<String> {
    blocks
        .iter()
        .filter_map(|b| match b {
            Block::Para { words, .. } => Some(text(words)),
            _ => None,
        })
        .collect()
}

const PROSE: &str = " Champa ist an sich nunmehr eine arme Fischersiedlung, wie Briggs es euch ja
 bereits erzählte. Die Monster sollten allerdings auch kein grös-
 seres Hindernis sein. Betretet also das Piratendorf und schaut euch um,
 denn es lohnt sich.

 Ein zweiter Absatz, der ebenso lang ist wie die Zeilen davor es waren, ja.
";

#[test]
fn hard_wrapped_lines_are_joined_into_paragraphs_and_split_words_made_whole() {
    let p = paras(&parse_txt(PROSE));
    assert_eq!(p.len(), 2, "{p:?}");
    assert!(p[0].contains("ja bereits erzählte"), "{}", p[0]);
    assert!(p[0].contains("kein grösseres Hindernis"), "{}", p[0]);
    assert!(p[0].ends_with("denn es lohnt sich."), "{}", p[0]);
}

#[test]
fn a_short_line_ends_its_paragraph() {
    let src = " Dies ist eine lange Zeile, die so lang ist wie die meisten Zeilen hier drin.
 Kurz.
 Dies ist eine lange Zeile, die so lang ist wie die meisten Zeilen hier drin.
 Dies ist eine lange Zeile, die so lang ist wie die meisten Zeilen hier drin.
";
    assert_eq!(paras(&parse_txt(src)).len(), 2);
}

#[test]
fn tables_and_drawings_keep_their_columns_and_their_underlined_title_is_a_heading() {
    let src = " Und das Angebot ist durchaus exquisit, das muss man der Dame lassen, ja.

  Händlerin
 ==========================
  Meister-Rapier    6800
  Große Axt         5200

 Ist der Kaufdrang erst einmal befriedigt, kann ganz in Ruhe erkundet werden.
";
    let blocks = parse_txt(src);
    assert!(
        blocks
            .iter()
            .any(|b| matches!(b, Block::Heading { words, .. } if text(words) == "Händlerin")),
        "{blocks:?}"
    );
    let pre = blocks.iter().find_map(|b| match b {
        Block::Pre { lines } => Some(lines.clone()),
        _ => None,
    });
    let pre = pre.expect("the table kept as it is");
    assert_eq!(pre[0].0, "Meister-Rapier    6800");
    assert_eq!(pre[1].0, "Große Axt         5200");
}

#[test]
fn a_decorated_line_is_a_heading() {
    let blocks = parse_txt("\n ~~ Champa ~~\n\n");
    assert!(matches!(&blocks[0], Block::Heading { words, .. } if text(words) == "Champa"));
}

#[test]
fn an_old_guide_in_the_windows_code_page_reads_right() {
    assert_eq!(
        decode(b"M\xe4rz \x93Ring\x94\r\n\tx"),
        "März “Ring”\n        x"
    );
}

#[test]
fn html_gives_headings_paragraphs_lists_and_preformatted_text_and_leaves_scripts_out() {
    let src = "<html><head><title>x</title><style>p{}</style></head><body>
<script>var a = '<p>no</p>';</script>
<h2>Kapitel &amp; Ende</h2><p>Ein <b>fetter</b> Satz,<br>gebrochen.</p>
<ul><li>eins</li><li>zwei</li></ul><pre>
A   B
1   2</pre><p>Gr&ouml;&szlig;e&#33;</p></body></html>";
    let blocks = parse_html(src);
    assert!(matches!(&blocks[0], Block::Heading { words, .. } if text(words) == "Kapitel & Ende"));
    let p = paras(&blocks);
    assert_eq!(p, vec!["Ein fetter Satz,", "gebrochen.", "Größe!"]);
    let items: Vec<_> = blocks
        .iter()
        .filter_map(|b| match b {
            Block::Item { words, mark, .. } => Some((mark.clone(), text(words))),
            _ => None,
        })
        .collect();
    assert_eq!(
        items,
        vec![("•".into(), "eins".into()), ("•".into(), "zwei".into())]
    );
    assert!(blocks
        .iter()
        .any(|b| matches!(b, Block::Pre { lines } if lines[0].0 == "A   B")));
    assert!(blocks
        .iter()
        .all(|b| !matches!(b, Block::Para { words, .. } if text(words).contains("no"))));
}

#[test]
fn a_place_in_the_guide_outlives_the_size_it_was_read_at() {
    let src: String = (0..200)
        .map(|i| format!(" Absatz {i} mit genug Worten, dass er über mehrere Zeilen läuft, wenn die Schrift groß ist.\n\n"))
        .collect();
    let blocks = parse(&src, false);
    let mut adv = Advances::default();
    let small = lay_out(&mut adv, &blocks, 600.0, 14.0);
    let large = lay_out(&mut adv, &blocks, 600.0, 26.0);
    assert!(large.height > small.height * 1.5);
    let at = small.src_at(small.lines[300].y);
    let back = large.src_at(large.y_of(at));
    assert!(back <= at, "{back} > {at}");
    let para = |s: usize| src[..s].matches("Absatz").count();
    assert_eq!(para(back), para(at), "landed in another paragraph");
}

#[test]
fn the_guide_takes_its_colours_from_the_theme() {
    use cartridge_ui::guide::GuideInk;
    use cartridge_ui::kit::palette::palette;
    let luma = |c: [u8; 3]| c[0] as u32 * 3 + c[1] as u32 * 6 + c[2] as u32;
    for colour in 0..8 {
        let ink = GuideInk::of(palette(colour));
        assert_eq!(ink.body, palette(colour).cream);
        assert!(
            luma(ink.body) > luma(ink.bg) * 4,
            "theme {colour}: text too faint"
        );
    }
    assert_ne!(GuideInk::of(palette(7)), GuideInk::of(palette(3)));
}
