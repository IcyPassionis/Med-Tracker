fn main() {
    println!("cargo:rerun-if-changed=native/x11_tray.c");
    println!("cargo:rerun-if-changed=build.rs");

    let gtk = pkg_config::Config::new()
        .atleast_version("3.0")
        .probe("gtk+-3.0")
        .expect("GTK 3 development files are required to build the X11 tray");

    cc::Build::new()
        .file("native/x11_tray.c")
        .includes(&gtk.include_paths)
        .warnings(true)
        .compile("med_tracker_x11_tray");

    // The C archive is linked before Rust's dependency objects. Repeat GTK's
    // link flags at the very end of the linker command so the references from
    // the static archive are resolved instead of being dropped by --as-needed.
    for path in &gtk.link_paths {
        println!("cargo:rustc-link-arg=-L{}", path.display());
    }
    for lib in &gtk.libs {
        println!("cargo:rustc-link-arg=-l{lib}");
    }
}
