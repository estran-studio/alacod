//! `alacod-gen <games/jeu> [--play] [--bless] …` : voir `scenario::cli` (le code est partagé avec le
//! binaire des jeux à plugin, `<jeu> scenarios gen`).

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(scenario::cli::run_gen(None, &args));
}
