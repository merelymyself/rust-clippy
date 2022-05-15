#![warn(clippy::path_from_format)]

use std::path::PathBuf;

fn main() {
    // let base_path = "";
    // PathBuf::from(format!("{}/foo/bar", base_path));
    let mut base_path1 = "";
    PathBuf::from(format!("{}/foo/bar", base_path1));
}
