//! Affiche un scénario : `cargo run -p scenario --features render --bin play_scenario -- <fichier.ron>`
//! (ou `make play_scenario SCENARIO=<nom>`).

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage : play_scenario <fichier.ron>");
    let source = std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{path} : {err}"));
    let scenario = scenario::Scenario::from_ron(&source).unwrap_or_else(|err| panic!("{path} : {err}"));
    scenario::runner::play(&scenario);
}
