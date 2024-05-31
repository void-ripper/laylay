use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub target: String,
}

impl Version {
    pub fn get() -> Version {
        Version {
            major: 0,
            minor: 1,
            patch: 0,
            target: "x86_64-unknown-linux-gnu".to_owned(),
        }
    }
    
    pub fn higher(&self, o: &Version) -> bool {
        self.major > o.major || self.minor > o.minor || self.patch > o.patch
    }
}
