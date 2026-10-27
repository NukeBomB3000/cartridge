//! What the shelf makes of Games/PS1: which files stand as games, which hide behind a playlist,
//! and the playlists made for games on several discs that have none.

use crate::common;
use crate::ps1::{cue_for, iso, pbp, raw, CNF};

use std::path::{Path, PathBuf};

use cartridge_store::ps1::{discs, PLAYLISTS_DIR};
use cartridge_store::{scan, Cart, Platform};
use common::tmp_root;
use tempfile::TempDir;

fn ps1(d: &TempDir) -> PathBuf {
    let dir = d.path().join("Games/PS1");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A cue sheet and its bin, `rel` being the cue's place under Games/PS1 without `.cue`. The
/// bin holds a tiny disc whose SYSTEM.CNF boots SLUS_005.94.
fn disc(d: &TempDir, rel: &str) -> PathBuf {
    let cue = ps1(d).join(format!("{rel}.cue"));
    std::fs::create_dir_all(cue.parent().unwrap()).unwrap();
    let bin = cue.with_extension("bin");
    std::fs::write(&bin, raw(&iso(CNF), 2)).unwrap();
    let bin_name = bin.file_name().unwrap().to_str().unwrap();
    std::fs::write(&cue, cue_for(bin_name, "MODE2/2352")).unwrap();
    cue
}

fn ps1_carts(d: &TempDir) -> Vec<Cart> {
    scan(d.path())
        .unwrap()
        .into_iter()
        .filter(|c| c.platform == Platform::Ps1)
        .collect()
}

fn stems(carts: &[Cart]) -> Vec<&str> {
    carts.iter().map(|c| c.stem.as_str()).collect()
}

fn playlist(d: &TempDir, stem: &str) -> PathBuf {
    d.path().join(PLAYLISTS_DIR).join(format!("{stem}.m3u"))
}

#[test]
fn a_cue_and_its_bin_are_one_game_and_a_bin_is_never_one() {
    let d = tmp_root();
    let cue = disc(&d, "Vagrant Story (USA)");
    std::fs::write(ps1(&d).join("Stray Track.bin"), [0u8; 16]).unwrap();
    let carts = ps1_carts(&d);
    assert_eq!(stems(&carts), ["Vagrant Story (USA)"]);
    assert_eq!(carts[0].rom, cue);
    assert_eq!(carts[0].code, "SLUS-00594", "the serial on the disc");
    assert!(
        !d.path().join(PLAYLISTS_DIR).exists(),
        "a lone disc needs no playlist"
    );
}

#[test]
fn a_chd_and_a_pbp_are_games_of_their_own() {
    let d = tmp_root();
    std::fs::write(ps1(&d).join("Ape Escape (USA).chd"), b"MComprHD").unwrap();
    std::fs::write(ps1(&d).join("Spyro (Europe).pbp"), pbp("SCES01438")).unwrap();
    std::fs::write(ps1(&d).join("Notes.txt"), b"hi").unwrap();
    std::fs::write(ps1(&d).join("Image.iso"), b"cooked").unwrap();
    let carts = ps1_carts(&d);
    assert_eq!(stems(&carts), ["Ape Escape (USA)", "Spyro (Europe)"]);
    assert_eq!(carts[0].code, "", "a chd's serial is not read");
    assert_eq!(carts[1].code, "SCES-01438");
}

#[test]
fn a_playlist_hides_the_discs_it_lists() {
    let d = tmp_root();
    for n in 1..=3 {
        disc(&d, &format!("Final Fantasy VII (USA) (Disc {n})"));
    }
    let m3u = ps1(&d).join("Final Fantasy VII (USA).m3u");
    std::fs::write(
        &m3u,
        "Final Fantasy VII (USA) (Disc 1).cue\n\
         ./Final Fantasy VII (USA) (Disc 2).cue\n\
         FINAL FANTASY VII (USA) (DISC 3).CUE\n",
    )
    .unwrap();
    let carts = ps1_carts(&d);
    assert_eq!(
        stems(&carts),
        ["Final Fantasy VII (USA)"],
        "a disc the playlist lists stood on the shelf as well"
    );
    assert_eq!(carts[0].rom, m3u);
    assert_eq!(carts[0].code, "SLUS-00594", "the first disc's serial");
    assert!(!d.path().join(PLAYLISTS_DIR).exists());
}

#[test]
fn a_playlist_that_lists_no_disc_there_is_not_a_game() {
    let d = tmp_root();
    std::fs::write(ps1(&d).join("Empty.m3u"), "# nothing\n").unwrap();
    std::fs::write(ps1(&d).join("Gone.m3u"), "Gone (Disc 1).cue\n").unwrap();
    assert!(ps1_carts(&d).is_empty());
}

#[test]
fn discs_with_no_playlist_are_one_game_with_one_made_for_them() {
    let d = tmp_root();
    let one = disc(&d, "Metal Gear Solid (USA) (Disc 1)");
    let two = disc(&d, "Metal Gear Solid (USA) (Disc 2)");
    disc(&d, "Vagrant Story (USA)");
    let carts = ps1_carts(&d);
    assert_eq!(
        stems(&carts),
        ["Metal Gear Solid (USA)", "Vagrant Story (USA)"]
    );
    let mgs = &carts[0];
    let m3u = playlist(&d, "Metal Gear Solid (USA)");
    assert_eq!(mgs.rom, m3u, "the game is not its made playlist");
    let text = std::fs::read_to_string(&m3u).unwrap();
    for line in text.lines() {
        assert!(Path::new(line).is_absolute(), "{line} is not a whole path");
    }
    assert_eq!(discs(&mgs.rom), [one, two], "the discs are not in order");
    assert_eq!(mgs.code, "SLUS-00594");
}

#[test]
fn the_made_playlist_follows_its_discs_and_goes_with_them() {
    let d = tmp_root();
    for n in [1, 2, 10] {
        disc(&d, &format!("Parasite Eve (USA) (Disc {n})"));
    }
    ps1_carts(&d);
    let m3u = playlist(&d, "Parasite Eve (USA)");
    let names = |m3u: &Path| -> Vec<String> {
        discs(m3u)
            .iter()
            .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
            .collect()
    };
    assert_eq!(
        names(&m3u),
        [
            "Parasite Eve (USA) (Disc 1)",
            "Parasite Eve (USA) (Disc 2)",
            "Parasite Eve (USA) (Disc 10)"
        ],
        "disc 10 is not after disc 2"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let before = std::fs::metadata(&m3u).unwrap().ino();
        ps1_carts(&d);
        let after = std::fs::metadata(&m3u).unwrap().ino();
        assert_eq!(before, after, "an unchanged playlist was written again");
    }

    std::fs::remove_file(ps1(&d).join("Parasite Eve (USA) (Disc 10).cue")).unwrap();
    ps1_carts(&d);
    assert_eq!(names(&m3u).len(), 2, "a disc taken away is still listed");

    for n in [1, 2] {
        std::fs::remove_file(ps1(&d).join(format!("Parasite Eve (USA) (Disc {n}).cue"))).unwrap();
    }
    assert!(ps1_carts(&d).is_empty());
    assert!(
        !m3u.exists(),
        "the playlist of a game taken away was left behind"
    );
}

#[test]
fn a_game_with_only_one_of_its_discs_is_that_disc_named_without_the_tag() {
    let d = tmp_root();
    let one = disc(&d, "Chrono Cross (USA) (Disc 1)");
    let carts = ps1_carts(&d);
    assert_eq!(stems(&carts), ["Chrono Cross (USA)"]);
    assert_eq!(carts[0].rom, one);
    assert!(!d.path().join(PLAYLISTS_DIR).exists());
}

#[test]
fn a_game_in_a_folder_of_its_own_is_found_but_not_two_folders_down() {
    let d = tmp_root();
    let cue = disc(&d, "Vagrant Story (USA)/Vagrant Story (USA)");
    for n in 1..=2 {
        disc(&d, &format!("FF8/Final Fantasy VIII (USA) (Disc {n})"));
    }
    disc(&d, "Xenogears/Disc 1/Xenogears (USA) (Disc 1)");
    disc(&d, ".Trashes/Deleted (USA)");
    let m3u_dir = ps1(&d).join("FF9");
    for n in 1..=2 {
        disc(&d, &format!("FF9/Final Fantasy IX (USA) (Disc {n})"));
    }
    std::fs::write(
        m3u_dir.join("Final Fantasy IX (USA).m3u"),
        "Final Fantasy IX (USA) (Disc 1).cue\nFinal Fantasy IX (USA) (Disc 2).cue\n",
    )
    .unwrap();
    let carts = ps1_carts(&d);
    assert_eq!(
        stems(&carts),
        [
            "Final Fantasy IX (USA)",
            "Final Fantasy VIII (USA)",
            "Vagrant Story (USA)"
        ]
    );
    assert_eq!(carts[0].rom, m3u_dir.join("Final Fantasy IX (USA).m3u"));
    assert_eq!(carts[1].rom, playlist(&d, "Final Fantasy VIII (USA)"));
    assert_eq!(carts[2].rom, cue);
}

#[test]
fn a_playlist_in_the_top_folder_hides_the_discs_it_lists_in_a_game_folder() {
    let d = tmp_root();
    for n in 1..=2 {
        disc(&d, &format!("Dino Crisis 2/Dino Crisis 2 (USA) (Disc {n})"));
    }
    std::fs::write(
        ps1(&d).join("Dino Crisis 2 (USA).m3u"),
        "Dino Crisis 2/Dino Crisis 2 (USA) (Disc 1).cue\n\
         Dino Crisis 2/Dino Crisis 2 (USA) (Disc 2).cue\n",
    )
    .unwrap();
    let carts = ps1_carts(&d);
    assert_eq!(stems(&carts), ["Dino Crisis 2 (USA)"]);
    assert!(!d.path().join(PLAYLISTS_DIR).exists());
}

#[test]
fn a_game_on_several_discs_has_its_own_label_or_else_its_first_discs() {
    let d = tmp_root();
    for n in 1..=2 {
        disc(&d, &format!("Metal Gear Solid (USA) (Disc {n})"));
    }
    disc(&d, "Vagrant Story (USA)");
    let labels = d.path().join("Labels/PS1");
    std::fs::create_dir_all(&labels).unwrap();
    std::fs::write(labels.join("Metal Gear Solid (USA) (Disc 1).png"), b"png").unwrap();
    std::fs::write(labels.join("Vagrant Story (USA).png"), b"png").unwrap();
    let carts = ps1_carts(&d);
    assert_eq!(
        carts[0].label.as_deref(),
        Some(labels.join("Metal Gear Solid (USA) (Disc 1).png").as_path())
    );
    assert_eq!(
        carts[1].label.as_deref(),
        Some(labels.join("Vagrant Story (USA).png").as_path())
    );
    std::fs::write(labels.join("Metal Gear Solid (USA).png"), b"png").unwrap();
    assert_eq!(
        ps1_carts(&d)[0].label.as_deref(),
        Some(labels.join("Metal Gear Solid (USA).png").as_path()),
        "the game's own label comes first"
    );
}

#[test]
fn the_other_systems_still_look_only_in_their_own_folder() {
    let d = tmp_root();
    std::fs::create_dir_all(d.path().join("Games/GBA/Hacks")).unwrap();
    std::fs::write(d.path().join("Games/GBA/Hacks/Deep.gba"), [0u8; 0x100]).unwrap();
    std::fs::write(d.path().join("Games/GBA/Top.gba"), [0u8; 0x100]).unwrap();
    let carts = scan(d.path()).unwrap();
    assert_eq!(stems(&carts), ["Top"]);
}
