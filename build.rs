//! Compiles the Windows resources into the executable.
//!
//! Without this, Explorer, the taskbar and Alt-Tab show the toolkit's generic
//! executable icon — the window icon `src/icon.rs` sets at startup is a
//! different thing, and does not reach the file. `assets/breadify.ico` is
//! written by `breadify::icon::ico()` and checked in; `tests/artwork.rs`
//! re-derives it and fails if the two have drifted.

fn main() {
    println!("cargo:rerun-if-changed=assets/breadify.ico");
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/breadify.ico");
        resource.set("ProductName", "Breadify");
        resource.set(
            "FileDescription",
            "Bread order exports into printed picking lists",
        );
        // Loud rather than quiet: a Windows build that silently loses its icon
        // is exactly what this file exists to prevent.
        resource
            .compile()
            .expect("the Windows icon resource should compile");
    }
}
