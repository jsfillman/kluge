#[test]
fn status_item_displays_kluge_menu() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../macos/KlugeMenu/main.swift");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(text.contains("NSStatusItem"), "{text}");
    assert!(text.contains("\"kluge\""), "{text}");
    assert!(text.contains("\"menu\""), "{text}");
    assert!(!text.contains("kluge cancel"), "{text}");
    assert!(!text.contains("kluge run"), "{text}");
    assert!(!text.contains("kluge sync"), "{text}");
}
