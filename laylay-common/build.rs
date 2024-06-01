fn main() {
    let code = format!(
        r#"use borsh::{{BorshDeserialize, BorshSerialize}};
use std::fmt::Display;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Version {{
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub target: String,
}}

impl Version {{
    pub fn get() -> Version {{
        Version {{
            major: {},
            minor: {},
            patch: {},
            target: "{}".to_owned(),
        }}
    }}
    
    pub fn higher(&self, o: &Version) -> bool {{
        self.major > o.major || self.minor > o.minor || self.patch > o.patch
    }}
}}


impl Display for Version {{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
        write!(
            f,
            "{{}}.{{}}.{{}} {{}}",
            self.major, self.minor, self.patch, self.target
        )
    }}
}}
"#,
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        std::env::var("TARGET").unwrap(),
    );

    std::fs::write("src/version.rs", code).unwrap();
}
