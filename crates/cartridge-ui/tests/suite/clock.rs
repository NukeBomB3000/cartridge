use cartridge_ui::kit::palette::palette;
use cartridge_ui::{clock_label, ClockPicker, Field, Polaroids};

/// The picker's line is flow.html's, a field to a box: where each field starts, in letters,
/// and how many it is.
const FIELDS: [(Field, u32, u32); 6] = [
    (Field::Year, 0, 4),
    (Field::Month, 7, 2),
    (Field::Day, 12, 2),
    (Field::Hour, 17, 2),
    (Field::Minute, 22, 2),
    (Field::Offset, 27, 9),
];

#[test]
fn the_picker_reads_as_flows_line() {
    let mut c = ClockPicker::from_ymd(2026, 10, 10);
    assert_eq!(c.line(), "2026 - 10 - 10   00 : 00   UTC+00:00");
    c.field(Field::Offset);
    c.up();
    assert_eq!(c.line(), "2026 - 10 - 10   00 : 00   UTC+00:30");
    assert_eq!(c.line().chars().count(), 36);
}

/// The underline is a pixel bar in the frame's bright colour on the line's row 11, under the
/// letters of the field the cursor is on and nowhere else, and it follows the cursor.
#[test]
fn the_underline_follows_the_field_the_cursor_is_on() {
    let pal = palette(0);
    let mut c = ClockPicker::from_ymd(2026, 10, 10);
    for (i, (field, start, letters)) in FIELDS.into_iter().enumerate() {
        assert_eq!(c.cursor(), field);
        let (x, w) = (6 * start, 6 * letters - 1);
        assert_eq!(c.underline(), [x, 11, w, 1], "{field:?}");
        let face = c.face(pal);
        let lit: Vec<u32> = (0..face.w)
            .filter(|&px| {
                let o = ((11 * face.w + px) * 4) as usize;
                face.rgba[o + 3] != 0
            })
            .collect();
        assert_eq!(lit, (x..x + w).collect::<Vec<_>>(), "{field:?}'s underline");
        let o = ((11 * face.w + x) * 4) as usize;
        assert_eq!(&face.rgba[o..o + 3], &pal.frame_hi);
        if i + 1 < FIELDS.len() {
            c.right();
        }
    }
    c.right();
    assert_eq!(c.cursor(), Field::Offset, "the cursor ran past the line");
    c.left();
    assert_eq!(c.underline()[0], 6 * 22, "Left went back to the minute");
}

#[test]
fn recent_states_read_relative_and_old_ones_read_absolute() {
    let now = "2026-08-09_14-36-05";
    assert_eq!(
        Polaroids::relative_time("2026-08-09_14-32-05", now),
        "4 min ago"
    );
    assert_eq!(
        Polaroids::relative_time("2026-08-09_03-00-00", now),
        "11 hr ago"
    );
    assert_eq!(
        Polaroids::relative_time("2026-08-08_20-00-00", now),
        "2026-08-08 20:00"
    );
    assert_eq!(
        Polaroids::relative_time("2025-01-02_09-05-00", now),
        "2025-01-02 09:05"
    );
}

#[test]
fn the_clock_is_24_hour_and_shows_no_seconds() {
    assert_eq!(
        clock_label("2026-08-09_21-07-00"),
        "21:07",
        "not 24 hour, or showing seconds"
    );
}
