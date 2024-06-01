use std::fmt::Display;

use borsh::{BorshDeserialize, BorshSerialize};
use uname::uname;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Info {
    pub sysname: String,
    pub nodename: String,
    pub release: String,
    pub version: String,
    pub machine: String,
}

impl Info {
    pub fn new() -> Self {
        let inf = uname().unwrap();

        Self {
            sysname: inf.sysname,
            nodename: inf.nodename,
            release: inf.release,
            version: inf.version,
            machine: inf.machine,
        }
    }
}

impl Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.sysname, self.nodename, self.release, self.version, self.machine
        )
    }
}
