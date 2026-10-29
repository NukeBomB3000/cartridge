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
fn the_folders_of_videos_are_tapes_of_their_own_and_take_only_videos() {
    use cartridge_store::media::{plays, tapes, Kind};
    let d = tempdir().unwrap();
    let videos = d.path().join("Videos");
    song(&videos.join("Series/Season 1"), "e01.mkv");
    song(&videos.join("Series/Season 1"), "cover.jpg");
    song(&videos.join("Film"), "film.MP4");
    song(&videos.join("Songs only"), "a.mp3");
    let found = tapes(Kind::Video, d.path(), &[]);
    assert_eq!(names(&found), ["Film", "Series"]);
    assert!(found.iter().all(|c| c.platform == Platform::Video));
    assert!(plays(Kind::Video, &videos.join("Film")));
    assert!(!plays(Kind::Video, &videos.join("Series")));
    // the music's tapes and the videos' do not mix: a song is no video
    assert!(tapes(Kind::Music, d.path(), &[]).is_empty());
}
