// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: MIT

fn main() {
    slint_build::compile("src/scene.slint").unwrap();
    println!("cargo:rustc-link-search=native=/opt/homebrew/Cellar/mpv/0.38.0_2/lib");
    println!("cargo:rustc-link-lib=mpv");
}
