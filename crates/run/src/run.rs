//! État de run (T2.4, chantier F1, `docs/plan-engine.md` §5 F1) : ressource rollback qui
//! représente la partie en cours — graine, mode, étape, joueurs — indépendamment du mode
//! (`Waves`, le système de vagues ; `Sandbox` ; `Floors`, séquence de niveaux livrée par T1.8,
//! `docs/conventions.md` §17 ; `Campaign` viendra en M2). Remplace `combat::downed::RunOutcome` (T1.3) : la défaite n'est plus une
//! ressource à part, c'est une valeur de [`RunStep`].
//!
//! [`Run`] est créée une seule fois par session, au démarrage (`game::jjrs::{local, p2p}`,
//! juste après `RunSeed`/`RngStreams`), avec la graine de la partie, le mode résolu du
//! manifeste (`content::manifest::EntryPoint::mode`, défaut `Waves` si le jeu déclare des
//! vagues, sinon `Sandbox`) et les handles GGRS des joueurs. Une relance (`RunRequest`,
//! `game::run`) en recrée une nouvelle avec la même configuration plutôt que de muter
//! celle-ci : un `Run` ne change jamais de graine ni de joueurs pendant sa vie.

use std::collections::BTreeSet;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Mode de la partie : quelles conditions de fin, quel résumé (voir
/// [`crate::modes::RunModeRules`]). `Waves` est le système de vagues actuel
/// (`game::waves`) ; `Floors` (T1.8, `docs/conventions.md` §17) est livré, `Campaign` viendra en
/// M2 (`docs/plan-engine.md` §5 F1) — c'est lui qui donnera un sens aux `flags` de [`Run`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RunMode {
    /// `config` : id de la config de vagues utilisée (nom de fichier sans extension,
    /// `content::registry::WaveConfigId`) — une seule aujourd'hui (`registry.waves` n'a
    /// jamais qu'une entrée en pratique) ; le champ prépare un choix explicite quand
    /// plusieurs configs seront possibles.
    Waves { config: String },
    /// Aucune condition de fin hors défaite : pour un jeu qui ne déclare pas de vagues
    /// (ex. `games/testbed`). Partie infinie jusqu'à ce que tous les joueurs tombent.
    Sandbox,
    /// Séquence de niveaux (T1.8) : `config` est l'id de la séquence (nom de fichier sans
    /// extension, `content::registry::FloorsConfigId`, contenu `Floors`). Portail quand le
    /// niveau n'a plus d'ennemi, boucle infinie au dernier niveau, pas de victoire : voir
    /// [`crate::floors`] et `docs/conventions.md` §17. Variante ajoutée **en dernier** : le
    /// hash dérivé de `RunMode` (discriminant) reste celui des variantes existantes, les
    /// traces des autres modes ne bougent pas.
    Floors { config: String },
}

/// Issue d'une partie terminée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RunEnd {
    /// Tous les joueurs à terre ou morts (T1.3, chantier B6) : universel, quel que soit le
    /// mode — voir `character::health::rollback_check_defeat`, hors de
    /// [`crate::modes::RunModeRules`].
    Defeat,
    /// Condition de victoire du mode atteinte (`RunModeRules::check_victory`).
    Victory,
    /// Le joueur a quitté vers le lobby avant la fin de la partie (`RunRequest::ToLobby`,
    /// `game::run`).
    Abandon,
}

/// Étape de la partie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum RunStep {
    /// Avant le début de la simulation. **Non atteint dans ce chantier** : [`Run`] n'est
    /// construite qu'au démarrage de session (voir la doc du module), directement
    /// `Playing` (voir [`Run::new`]) — réservé à un futur état où `Run` existerait déjà
    /// pendant le lobby (M1/M2, ex. matchmaking). Valeur par défaut de l'enum (utile aux
    /// tests), jamais celle d'un `Run` réellement inséré en ressource.
    #[default]
    Lobby,
    /// En cours depuis la frame `since_frame`.
    Playing { since_frame: u32 },
    /// Terminée à la frame `at_frame`, avec son issue. Ne change plus jamais une fois
    /// posée (`rollback_check_defeat`/le système de victoire du mode/`RunRequest::ToLobby`
    /// vérifient tous `Run::is_playing` avant d'écrire).
    Ended { at_frame: u32, outcome: RunEnd },
}

/// Résumé d'une partie terminée (T2.4) : calculé une seule fois, à `Ended`
/// (`RunModeRules::summarize`), conservé dans [`Run::summary`]. Affiché par l'écran de fin
/// (`game::ui::game_over`), exposé par `scenario::events::GameEvents::summary` et par les
/// attentes de scénario `RunState`/`RunSummary` (`game::replay::Expectation`).
///
/// `Hash` est écrit à la main (voir son impl) : `floor_reached` (T1.8) n'entre dans le hash
/// que s'il est non nul, pour que le checksum d'un résumé des autres modes reste celui
/// d'avant T1.8 au bit près.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSummary {
    /// Vague atteinte (mode `Waves` ; `0` pour un mode sans notion de vague).
    pub wave_reached: u32,
    /// Ennemis tués au total (`WaveState::total_enemies_killed`, mode `Waves`).
    pub kills: u32,
    /// Somme des soldes (`Currency`) de tous les joueurs à la fin de la partie.
    pub points_total: u32,
    /// Frame à laquelle la partie s'est terminée (`RunStep::Ended::at_frame`).
    pub frames: u32,
    pub outcome: RunEnd,
    /// Niveau atteint (mode `Floors`, T1.8 : `FloorState::index`, `0` = premier niveau) ;
    /// toujours `0` pour les autres modes.
    #[serde(default)]
    pub floor_reached: u32,
}

impl std::hash::Hash for RunSummary {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Mêmes champs, même ordre que l'ancien `#[derive(Hash)]` : hash inchangé tant que
        // `floor_reached == 0` (tous les modes sauf `Floors`).
        self.wave_reached.hash(state);
        self.kills.hash(state);
        self.points_total.hash(state);
        self.frames.hash(state);
        self.outcome.hash(state);
        if self.floor_reached != 0 {
            self.floor_reached.hash(state);
        }
    }
}

/// État de run : ressource rollback (checksum GGRS + trace d'état, voir `RunPlugin` dans
/// `lib.rs`) — voir la doc du module pour sa création et son cycle de vie.
///
/// **Décision** (documentée dans le rapport de la tâche) : [`RunSummary`] est un champ de
/// `Run` (`summary: Option<RunSummary>`) plutôt qu'une ressource rollback séparée — un seul
/// enregistrement (`rollback_and_trace_resource::<Run>()`), pas de valeur sentinelle à
/// inventer pour « pas encore de résumé » (`RunSummary::outcome: RunEnd` n'a pas de valeur
/// neutre, contrairement à l'ancien `RunOutcome::defeat_at_frame: Option<u32>`).
#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Run {
    /// Graine de la partie (`bevy_fixed::rng::RunSeed::0` au moment de la création),
    /// dupliquée ici pour que `Run` seule décrive entièrement la partie (résumé, relance
    /// avec « la même configuration ») sans avoir à lire une autre ressource.
    pub seed: u32,
    pub mode: RunMode,
    pub step: RunStep,
    /// Handles GGRS des joueurs de cette partie, triés.
    pub players: Vec<usize>,
    /// Drapeaux ouverts pour de futurs modes/contenus (M1/M2) ; vide aujourd'hui, aucun
    /// système n'y écrit ni n'y lit.
    pub flags: BTreeSet<String>,
    /// Résumé, rempli une seule fois à `Ended` (voir la doc du champ ci-dessus et
    /// `RunModeRules::summarize`). `None` tant que `step` n'est pas `Ended`.
    pub summary: Option<RunSummary>,
}

impl Run {
    /// Nouvelle partie « en cours » depuis `since_frame` (`players` est trié). Voir la doc
    /// du module : `Run` naît toujours directement `Playing`, jamais `Lobby`.
    pub fn new(seed: u32, mode: RunMode, mut players: Vec<usize>, since_frame: u32) -> Self {
        players.sort_unstable();
        Self {
            seed,
            mode,
            step: RunStep::Playing { since_frame },
            players,
            flags: BTreeSet::new(),
            summary: None,
        }
    }

    /// Vrai si la partie est en cours (ni en lobby, ni terminée). C'est la seule étape où
    /// les systèmes de fin de partie (défaite T1.3, victoire du mode, `RunRequest::ToLobby`)
    /// doivent écrire dans `step` — tous vérifient ce prédicat avant d'écrire, pour ne
    /// jamais remplacer une issue déjà posée (« une seule fois », voir la doc de `RunStep`).
    pub fn is_playing(&self) -> bool {
        matches!(self.step, RunStep::Playing { .. })
    }

    pub fn is_ended(&self) -> bool {
        matches!(self.step, RunStep::Ended { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_run_starts_playing_with_sorted_players() {
        let run = Run::new(42, RunMode::Sandbox, vec![2, 0, 1], 7);
        assert_eq!(run.step, RunStep::Playing { since_frame: 7 });
        assert_eq!(run.players, vec![0, 1, 2]);
        assert!(run.summary.is_none());
        assert!(run.flags.is_empty());
        assert!(run.is_playing());
        assert!(!run.is_ended());
    }

    #[test]
    fn ended_run_is_not_playing() {
        let mut run = Run::new(1, RunMode::Sandbox, vec![0], 0);
        run.step = RunStep::Ended {
            at_frame: 10,
            outcome: RunEnd::Defeat,
        };
        assert!(!run.is_playing());
        assert!(run.is_ended());
    }
}
