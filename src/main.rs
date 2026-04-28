fn main() {
    let response = packvet::cli::run_interactive(std::env::args());

    print!("{}", response.stdout);
    eprint!("{}", response.stderr);

    std::process::exit(response.exit_code);
}
