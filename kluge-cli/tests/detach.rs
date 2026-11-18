use kluge_cli::{install_completion, on_interrupt, LogFollow};

#[test]
fn install_writes_three_shells() {
    let home = std::env::temp_dir().join(format!("kluge-home-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    install_completion(&home).unwrap();
    for shell in ["bash", "zsh", "fish"] {
        let path = home.join(".kluge").join(format!("completion.{shell}"));
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("kluge"), "{text}");
        assert!(text.contains("complete"), "{text}");
        assert!(text.contains(shell), "{text}");
    }
    let _ = std::fs::remove_dir_all(home);
}

#[test]
fn interrupt_stops_without_cancel() {
    let mut follow = LogFollow {
        stop: false,
        cancelled: false,
    };
    on_interrupt(&mut follow);
    assert!(follow.stop);
    assert!(!follow.cancelled);
}
