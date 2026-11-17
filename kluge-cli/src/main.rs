fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (code, text) = kluge_cli::main_args(&args);
    print!("{text}");
    if !text.is_empty() && !text.ends_with('\n') {
        println!();
    }
    std::process::exit(code);
}
