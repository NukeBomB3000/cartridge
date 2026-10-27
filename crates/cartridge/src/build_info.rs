#[derive(Copy, Clone, Debug)]
pub struct Build {
    pub version: &'static str,
    pub hash: &'static str,
    pub dirty: bool,
    pub date: &'static str,
}

impl Build {
    pub fn current() -> Build {
        Build {
            version: env!("CARGO_PKG_VERSION"),
            hash: env!("CARTRIDGE_GIT_HASH"),
            dirty: env!("CARTRIDGE_GIT_DIRTY") == "1",
            date: env!("CARTRIDGE_BUILD_DATE"),
        }
    }

    pub fn serial(&self) -> String {
        self.hash.to_uppercase()
    }

    /// The build as About's BUILD row shows it: the commit's short hash in capitals, and
    /// `-DIRTY` after it for a build made with changes not yet committed, as Git's own
    /// `describe --dirty` marks one.
    pub fn shown(&self) -> String {
        match self.dirty {
            true => format!("{}-DIRTY", self.serial()),
            false => self.serial(),
        }
    }
}
