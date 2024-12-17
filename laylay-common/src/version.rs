use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt::Display;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub branch: String,
    pub commit: String,
    pub target: String,
}

impl Version {
    pub fn get() -> Version {
        Version {
            major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
            patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
            branch: env!("BRANCH").into(),
            commit: env!("COMMIT").into(),
            target: env!("TARGET").into(),
        }
    }

    pub fn higher(&self, o: &Version) -> bool {
        self.major > o.major
            || self.major == o.major && self.minor > o.minor
            || self.major == o.major && self.minor == o.minor && self.patch > o.patch
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{} {}",
            self.major, self.minor, self.patch, self.target
        )
    }
}
