//! Temps par set de systèmes du rollback (`ALACOD_SYSTEM_PROFILE=1`) : répond à « où passe
//! la frame ? » quand le profil du rollback (`snapshot_profile`) montre une frame de simulation
//! trop chère. Un système-frontière entre deux sets consécutifs (ordonnés en chaîne) clôt le
//! temps du précédent et ouvre celui du suivant : avec `ambiguity_detection: Error`, c'est la
//! seule forme qui n'introduit aucune ambiguïté d'ordre (une frontière par paire de voisins).
//!
//! Diagnostic seulement : sans la variable, aucun système n'est enregistré (voir
//! `game::core`, qui pose les frontières sur `RollbackSystemSet::ORDER`). Les temps ne
//! touchent ni la simulation ni le checksum.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use bevy::prelude::*;

/// `ALACOD_SYSTEM_PROFILE=1` : poser les frontières et imprimer le rapport à la sortie.
pub fn requested() -> bool {
    std::env::var("ALACOD_SYSTEM_PROFILE").is_ok_and(|v| v == "1")
}

#[derive(Resource, Default, Debug)]
pub struct SetTimings {
    started: Option<(&'static str, Instant)>,
    totals: BTreeMap<&'static str, (u64, Duration)>,
}

impl SetTimings {
    fn boundary(&mut self, next: Option<&'static str>) {
        let now = Instant::now();
        if let Some((name, at)) = self.started.take() {
            let entry = self.totals.entry(name).or_insert((0, Duration::ZERO));
            entry.0 += 1;
            entry.1 += now.duration_since(at);
        }
        self.started = next.map(|name| (name, now));
    }

    /// Les sets par temps cumulé décroissant : `total ms | frames | moyenne µs | set`.
    pub fn report(&self) -> Vec<String> {
        let mut rows: Vec<(&&str, &(u64, Duration))> = self.totals.iter().collect();
        rows.sort_by(|a, b| b.1 .1.cmp(&a.1 .1).then(a.0.cmp(b.0)));
        rows.iter()
            .map(|(name, (count, total))| {
                format!(
                    "{:>9.1} ms | {:>6} frames | {:>8.1} µs | {}",
                    total.as_secs_f64() * 1000.0,
                    count,
                    total.as_secs_f64() * 1_000_000.0 / (*count).max(1) as f64,
                    name
                )
            })
            .collect()
    }
}

/// Système-frontière : clôt le set en cours (s'il y en a un) et ouvre `next` (`None` à la fin
/// de la chaîne). À ordonner `.after(précédent).before(next)` par l'appelant. Sans la
/// ressource (profil non demandé), ne fait rien.
pub fn boundary(next: Option<&'static str>) -> impl Fn(Option<ResMut<SetTimings>>) {
    move |timings| {
        if let Some(mut timings) = timings {
            timings.boundary(next);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundaries_attribute_time_to_the_closed_set() {
        let mut timings = SetTimings::default();
        timings.boundary(Some("a"));
        std::thread::sleep(Duration::from_millis(2));
        timings.boundary(Some("b"));
        timings.boundary(None);
        assert_eq!(timings.totals["a"].0, 1);
        assert!(timings.totals["a"].1 >= Duration::from_millis(2));
        assert_eq!(timings.totals["b"].0, 1);
        assert!(timings.started.is_none());
        assert!(timings.report()[0].ends_with("| a"));
    }
}
