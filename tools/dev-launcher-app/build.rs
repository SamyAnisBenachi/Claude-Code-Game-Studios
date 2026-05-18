// build.rs -- embed the CCGS Dev Launcher icon as the EXE's Win32 ICON
// resource. Windows Explorer, the taskbar, Alt-Tab, and "Open With..." all
// pick up the lowest-numbered ICON resource in the EXE image, so embedding
// here makes the launcher recognisable everywhere the OS surfaces it -- not
// just inside the launcher's own window.
//
// The runtime window-icon assignment lives in src/main.rs (the same .ico
// bytes are bundled via `include_bytes!` and handed to nwg::Icon). The two
// paths are intentionally redundant: the resource path covers the OS shell,
// the runtime path covers the in-process window title bar / task bar group.
//
// No-op on non-Windows targets so `cargo check` / `cargo doc --target ...`
// from other hosts still succeed.

fn main() {
    // Rebuild only when the icon or this script change. Resource compilation
    // is fast but unnecessary on every gameplay-code touch.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=res/ccgs-dev-launcher.ico");

    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("res/ccgs-dev-launcher.ico");
        // Surfaces in Explorer's "Details" pane on the EXE.
        res.set("FileDescription", "CCGS Dev Launcher");
        res.set("ProductName", "CCGS Dev Launcher");
        res.set("OriginalFilename", "ccgs-dev-launcher.exe");
        res.set("CompanyName", "Claude Code Game Studios");
        res.set(
            "LegalCopyright",
            "Internal CCGS development tool. Not for redistribution.",
        );
        if let Err(e) = res.compile() {
            // Fail loudly so a missing/locked rc.exe surfaces during the
            // documented build flow rather than silently shipping an
            // unbranded EXE.
            panic!("embedding launcher icon resource failed: {e}");
        }
    }
}
