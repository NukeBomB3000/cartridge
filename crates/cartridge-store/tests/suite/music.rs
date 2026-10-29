use std::fs;
use std::path::Path;

use cartridge_store::music::{contents, folder, plays, songs, tapes};
use cartridge_store::Platform;
use tempfile::tempdir;

fn song(dir: &Path, name: &str) {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join(name), b"x").unwrap();
}

fn names(carts: &[cartridge_store::Cart]) -> Vec<&str> {
    carts.iter().map(|c| c.stem.as_str()).collect()
}

#[test]
fn the_folders_of_music_are_its_tapes_in_the_order_the_shelf_sorts_games() {
    let d = tempdir().unwrap();
    let music = d.path().join("Music");
    song(&music.join("Zebra/2001 - First"), "a.mp3");
    song(&music.join("abba"), "b.flac");
    song(&music.join("10cc"), "c.ogg");
    let found = tapes(d.path(), &[]);
    assert_eq!(names(&found), ["10cc", "abba", "Zebra"]);
    assert!(found.iter().all(|c| c.platform == Platform::Music));
}

#[test]
fn a_folder_with_folders_is_opened_and_one_with_only_songs_is_played() {
    let d = tempdir().unwrap();
    let music = d.path().join("Music");
    song(&music.join("Band/2001 - First"), "a.mp3");
    song(&music.join("Band/2002 - Second"), "b.mp3");
    assert!(!plays(&music.join("Band")));
    assert!(plays(&music.join("Band/2001 - First")));
    let inside = tapes(d.path(), &["Band".to_string()]);
    assert_eq!(names(&inside), ["2001 - First", "2002 - Second"]);
    assert_eq!(folder(d.path(), &["Band".to_string()]), music.join("Band"));
}

#[test]
fn folders_with_no_songs_in_them_are_not_tapes() {
    let d = tempdir().unwrap();
    let music = d.path().join("Music");
    song(&music.join("Empty/Deeper"), "notes.txt");
    song(&music.join("Real"), "a.m4a");
    fs::create_dir_all(music.join(".hidden")).unwrap();
    song(&music.join(".hidden"), "a.mp3");
    assert_eq!(names(&tapes(d.path(), &[])), ["Real"]);
}

#[test]
fn songs_are_listed_in_name_order_whatever_their_case_or_extension() {
    let d = tempdir().unwrap();
    let album = d.path().join("Music/Album");
    for name in [
        "10 - Last.mp3",
        "02 - Two.MP3",
        "01 - One.flac",
        "cover.jpg",
    ] {
        song(&album, name);
    }
    let listed: Vec<String> = songs(&album)
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    let listed: Vec<&str> = listed.iter().map(String::as_str).collect();
    assert_eq!(listed, ["01 - One.flac", "02 - Two.MP3", "10 - Last.mp3"]);
    assert_eq!(contents(&album).folders.len(), 0);
}

#[test]
fn songs_loose_in_music_make_one_tape_of_the_folder_itself() {
    let d = tempdir().unwrap();
    song(&d.path().join("Music"), "a.mp3");
    song(&d.path().join("Music"), "b.mp3");
    let found = tapes(d.path(), &[]);
    assert_eq!(names(&found), ["Music"]);
    assert!(plays(&found[0].rom));
}

#[test]
fn a_card_with_no_music_has_no_tapes() {
    let d = tempdir().unwrap();
    assert!(tapes(d.path(), &[]).is_empty());
}

#[test]
fn a_video_is_a_tape_of_its_own_and_a_folder_of_one_plays_it() {
    use cartridge_store::media::{files, plays, tapes, Kind};
    let d = tempdir().unwrap();
    let videos = d.path().join("Videos");
    song(&videos.join("Series/Season 1"), "e01.mkv");
    song(&videos.join("Series/Season 1"), "e02.mkv");
    song(&videos.join("Series/Season 1"), "cover.jpg");
    song(&videos.join("Film"), "film.MP4");
    song(&videos, "Loose Film.avi");
    song(&videos.join("Songs only"), "a.mp3");
    let found = tapes(Kind::Video, d.path(), &[]);
    // the folders first, then the videos lying loose
    assert_eq!(names(&found), ["Film", "Series", "Loose Film"]);
    assert!(found.iter().all(|c| c.platform == Platform::Video));
    assert!(
        plays(Kind::Video, &videos.join("Film")),
        "a folder of one video plays it"
    );
    assert!(plays(Kind::Video, &videos.join("Loose Film.avi")));
    assert!(!plays(Kind::Video, &videos.join("Series")));
    // a season's episodes are a tape each, and none of them is a folder to open
    let season = tapes(
        Kind::Video,
        d.path(),
        &["Series".to_string(), "Season 1".to_string()],
    );
    assert_eq!(names(&season), ["e01", "e02"]);
    assert_eq!(files(Kind::Video, &videos.join("Loose Film.avi")).len(), 1);
    // the music's tapes and the videos' do not mix: a song is no video
    assert!(tapes(Kind::Music, d.path(), &[]).is_empty());
}

#[test]
fn pictures_loose_in_their_folder_are_one_polaroid_wearing_the_first() {
    use cartridge_store::media::{plays, tapes, Kind};
    let d = tempdir().unwrap();
    let pictures = d.path().join("Pictures");
    for name in [
        "Birne.jpg",
        "Makro-4190.jpg",
        "Norman Avatar-7.jpg",
        "notes.txt",
    ] {
        song(&pictures, name);
    }
    let found = tapes(Kind::Pictures, d.path(), &[]);
    assert_eq!(names(&found), ["Pictures"]);
    assert_eq!(found[0].platform, Platform::Pictures);
    assert!(found[0]
        .label
        .as_ref()
        .is_some_and(|l| l.ends_with("Birne.jpg")));
    assert!(plays(Kind::Pictures, &pictures));
}

#[test]
fn a_folder_of_folders_of_pictures_has_a_polaroid_for_each_wearing_its_first() {
    use cartridge_store::media::{tapes, Kind};
    let d = tempdir().unwrap();
    song(&d.path().join("Pictures/Holiday/Day 1"), "a.png");
    song(&d.path().join("Pictures/Family"), "b.JPEG");
    let found = tapes(Kind::Pictures, d.path(), &[]);
    assert_eq!(names(&found), ["Family", "Holiday"]);
    assert!(found[1]
        .label
        .as_ref()
        .is_some_and(|l| l.ends_with("a.png")));
}
