fn main() {
    tauri_build::build();

    // Embed Info.plist into the binary for macOS 26 compatibility.
    // macOS 26+ requires apps to have a bundle identifier (from Info.plist).
    // Without this, macOS throws an Objective-C exception:
    //   "[WindowTab] Cannot index window tabs due to missing main bundle identifier"
    #[cfg(target_os = "macos")]
    {
        let plist_path = std::path::Path::new("Info.plist");
        if plist_path.exists() {
            println!(
                "cargo:rustc-link-arg=-Wl,-sectcreate,__TEXT,__info_plist,{}",
                plist_path.canonicalize().unwrap().display()
            );
        }
    }
}
