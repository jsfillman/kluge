fn main() {
    install_sigint();
    let args: Vec<String> = std::env::args().collect();
    let api = std::env::var("KLUGE_API").ok();
    let (code, text) = kluge_cli::main_with(&args, api.as_deref());
    print!("{text}");
    if !text.is_empty() && !text.ends_with('\n') {
        println!();
    }
    std::process::exit(code);
}

static INTERRUPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

extern "C" fn handle_sigint(_: i32) {
    INTERRUPTED.store(true, std::sync::atomic::Ordering::SeqCst);
}

fn install_sigint() {
    unsafe {
        signal(2, handle_sigint as usize);
    }
}

unsafe extern "C" {
    fn signal(signum: i32, handler: usize) -> usize;
}
