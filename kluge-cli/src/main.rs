fn main() {
    let args: Vec<String> = std::env::args().collect();
    let api = std::env::var("KLUGE_API").ok();
    let (code, text) = kluge_cli::main_with(&args, api.as_deref());
    print!("{text}");
    if !text.is_empty() && !text.ends_with('\n') {
        println!();
    }
    std::process::exit(code);
}
