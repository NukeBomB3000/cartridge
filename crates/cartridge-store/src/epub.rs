//! An EPUB: a zip with a container file naming the package (OPF), which names the book, the
//! files that make it up, the order they are read in and the cover. Only what the shelf and the
//! reader need is read: the title, the author, the cover picture and the chapters' markup.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use zip::ZipArchive;

pub struct Epub {
    zip: ZipArchive<File>,
    /// The folder of the package file inside the zip, with a `/` after it, or nothing.
    base: String,
    pub title: String,
    pub author: String,
    /// The chapters, as the paths of their files inside the zip, in the order they are read.
    spine: Vec<String>,
    /// The cover picture's path inside the zip.
    cover: Option<String>,
}

/// `%20` and the like in a path inside the zip, as the file is named.
fn unescape(href: &str) -> String {
    let bytes = href.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() + 0 {
            if let Ok(v) = u8::from_str_radix(&href[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `href` as a path inside the zip, from the package file's folder `base`, with `..` and `.`
/// settled and any `#fragment` dropped.
fn resolve(base: &str, href: &str) -> String {
    let href = unescape(href.split('#').next().unwrap_or_default());
    let mut parts: Vec<&str> = base.split('/').filter(|p| !p.is_empty()).collect();
    for part in href.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    parts.join("/")
}

fn read_entry(zip: &mut ZipArchive<File>, name: &str) -> Option<Vec<u8>> {
    let mut entry = zip.by_name(name).ok()?;
    let mut bytes = Vec::with_capacity(entry.size().min(64 << 20) as usize);
    entry.read_to_end(&mut bytes).ok()?;
    Some(bytes)
}

impl Epub {
    pub fn open(path: &Path) -> Option<Epub> {
        let mut zip = ZipArchive::new(File::open(path).ok()?).ok()?;
        let container = String::from_utf8(read_entry(&mut zip, "META-INF/container.xml")?).ok()?;
        let doc = roxmltree::Document::parse(&container).ok()?;
        let opf_path = doc
            .descendants()
            .find(|n| n.has_tag_name("rootfile"))?
            .attribute("full-path")?
            .to_string();
        let opf_text = String::from_utf8_lossy(&read_entry(&mut zip, &opf_path)?).into_owned();
        let opf = roxmltree::Document::parse(&opf_text).ok()?;
        let base = match opf_path.rsplit_once('/') {
            Some((dir, _)) => format!("{dir}/"),
            None => String::new(),
        };
        let text_of = |tag: &str| {
            opf.descendants()
                .find(|n| n.tag_name().name() == tag)
                .and_then(|n| n.text())
                .map(|t| t.trim().to_string())
                .unwrap_or_default()
        };
        let (title, author) = (text_of("title"), text_of("creator"));
        // the manifest: id, href, type and properties of each file
        let items: Vec<(String, String, String, String)> = opf
            .descendants()
            .filter(|n| n.tag_name().name() == "item")
            .map(|n| {
                (
                    n.attribute("id").unwrap_or_default().to_string(),
                    n.attribute("href").unwrap_or_default().to_string(),
                    n.attribute("media-type").unwrap_or_default().to_string(),
                    n.attribute("properties").unwrap_or_default().to_string(),
                )
            })
            .collect();
        let by_id = |id: &str| items.iter().find(|i| i.0 == id);
        let spine: Vec<String> = opf
            .descendants()
            .filter(|n| n.tag_name().name() == "itemref")
            .filter(|n| n.attribute("linear") != Some("no"))
            .filter_map(|n| by_id(n.attribute("idref")?))
            .map(|i| resolve(&base, &i.1))
            .collect();
        let is_image = |i: &&(String, String, String, String)| i.2.starts_with("image/");
        let cover_id = opf
            .descendants()
            .find(|n| n.tag_name().name() == "meta" && n.attribute("name") == Some("cover"))
            .and_then(|n| n.attribute("content"));
        let cover = items
            .iter()
            .find(|i| i.3.split_whitespace().any(|p| p == "cover-image"))
            .or_else(|| cover_id.and_then(by_id))
            .or_else(|| {
                items.iter().filter(is_image).find(|i| {
                    let name = format!("{} {}", i.0, i.1).to_ascii_lowercase();
                    name.contains("cover")
                })
            })
            .map(|i| resolve(&base, &i.1));
        Some(Epub {
            zip,
            base,
            title,
            author,
            spine,
            cover,
        })
    }

    /// The cover's file extension and bytes, if the book has a cover picture.
    pub fn cover(&mut self) -> Option<(String, Vec<u8>)> {
        let name = self.cover.clone()?;
        let ext = name
            .rsplit('.')
            .next()
            .unwrap_or("jpg")
            .to_ascii_lowercase();
        Some((ext, read_entry(&mut self.zip, &name)?))
    }

    /// How many chapters (files read one after the other) the book has.
    pub fn chapters(&self) -> usize {
        self.spine.len()
    }

    /// Chapter `n`'s markup, as text.
    pub fn chapter(&mut self, n: usize) -> Option<String> {
        let name = self.spine.get(n)?.clone();
        Some(String::from_utf8_lossy(&read_entry(&mut self.zip, &name)?).into_owned())
    }

    /// The folder the package file is in, inside the zip.
    pub fn base(&self) -> &str {
        &self.base
    }
}

/// The folder covers cut from books are kept in, under the card's root: hidden, so no scan or
/// shelf lists it.
const COVERS: &str = "Labels/Books/.covers";

/// The cover of the book at `epub` as a file on the card, cut from it the first time and kept:
/// `Labels/Books/.covers/<book>.<ext>`. None for a book with no cover, or one that cannot be read.
pub fn cached_cover(root: &Path, epub: &Path) -> Option<PathBuf> {
    let stem = epub.file_stem()?.to_str()?;
    let dir = root.join(COVERS);
    for ext in ["jpg", "jpeg", "png"] {
        let kept = dir.join(format!("{stem}.{ext}"));
        if kept.is_file() {
            return Some(kept);
        }
    }
    let (ext, bytes) = Epub::open(epub)?.cover()?;
    if !matches!(ext.as_str(), "jpg" | "jpeg" | "png") {
        return None;
    }
    std::fs::create_dir_all(&dir).ok()?;
    let to = dir.join(format!("{stem}.{ext}"));
    std::fs::write(&to, bytes).ok()?;
    Some(to)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    fn book(dir: &Path, name: &str, opf: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let path = dir.join(name);
        let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
        let opts = SimpleFileOptions::default();
        zip.start_file("META-INF/container.xml", opts).unwrap();
        zip.write_all(br#"<?xml version="1.0"?><container><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#).unwrap();
        zip.start_file("OEBPS/content.opf", opts).unwrap();
        zip.write_all(opf.as_bytes()).unwrap();
        for (name, bytes) in files {
            zip.start_file(*name, opts).unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap();
        path
    }

    const OPF: &str = r#"<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0">
 <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:title>Nineteen Eighty-Four</dc:title><dc:creator>George Orwell</dc:creator>
  <meta name="cover" content="cov"/>
 </metadata>
 <manifest>
  <item id="cov" href="images/cover%20art.png" media-type="image/png"/>
  <item id="c1" href="text/one.xhtml" media-type="application/xhtml+xml"/>
  <item id="c2" href="text/two.xhtml" media-type="application/xhtml+xml"/>
  <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml"/>
 </manifest>
 <spine><itemref idref="c1"/><itemref idref="nav" linear="no"/><itemref idref="c2"/></spine>
</package>"#;

    #[test]
    fn a_book_gives_its_title_author_cover_and_chapters_in_reading_order() {
        let d = tempfile::tempdir().unwrap();
        let path = book(
            d.path(),
            "1984.epub",
            OPF,
            &[
                ("OEBPS/images/cover art.png", b"PNGDATA"),
                ("OEBPS/text/one.xhtml", b"<p>One</p>"),
                ("OEBPS/text/two.xhtml", b"<p>Two</p>"),
            ],
        );
        let mut epub = Epub::open(&path).expect("an epub");
        assert_eq!(epub.title, "Nineteen Eighty-Four");
        assert_eq!(epub.author, "George Orwell");
        assert_eq!(
            epub.chapters(),
            2,
            "a chapter marked not linear is left out"
        );
        assert_eq!(epub.chapter(0).unwrap(), "<p>One</p>");
        assert_eq!(epub.chapter(1).unwrap(), "<p>Two</p>");
        let (ext, bytes) = epub.cover().expect("a cover");
        assert_eq!((ext.as_str(), bytes.as_slice()), ("png", &b"PNGDATA"[..]));
    }

    #[test]
    fn the_cover_is_cut_to_the_card_once_and_kept() {
        let d = tempfile::tempdir().unwrap();
        let path = book(
            d.path(),
            "A Book.epub",
            OPF,
            &[("OEBPS/images/cover art.png", b"PNGDATA")],
        );
        let cut = cached_cover(d.path(), &path).expect("a cover");
        assert!(cut.ends_with("Labels/Books/.covers/A Book.png"));
        assert_eq!(std::fs::read(&cut).unwrap(), b"PNGDATA");
        assert_eq!(cached_cover(d.path(), &path), Some(cut));
    }

    #[test]
    fn a_file_that_is_no_book_does_not_open() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("x.epub");
        std::fs::write(&path, b"not a zip").unwrap();
        assert!(Epub::open(&path).is_none());
        assert!(cached_cover(d.path(), &path).is_none());
    }

    #[test]
    fn paths_inside_the_zip_are_settled() {
        assert_eq!(
            resolve("OEBPS/", "text/one.xhtml#a"),
            "OEBPS/text/one.xhtml"
        );
        assert_eq!(
            resolve("OEBPS/text/", "../img/c%20d.png"),
            "OEBPS/img/c d.png"
        );
        assert_eq!(resolve("", "./a.xhtml"), "a.xhtml");
    }
}
