//! Rechargement à chaud du registre de contenu, dans tous les états (T1.5, K5-1).
//!
//! Le registre ([`content::registry::Registry`], et le manifeste [`content::manifest::GameManifest`])
//! est relu depuis le disque (`load_and_lint` sur [`GameRoot`]) quand :
//! - un asset Bevy sentinelle émet `AssetEvent::Modified` (personnages, armes, mêlée, vagues ;
//!   feature `native`, `bevy/file_watcher`) ;
//! - la **scrutation** (natif seulement) voit changer l'empreinte des fichiers du contenu : (chemin
//!   relatif, mtime, taille) de `game.ron` et de chaque fichier ou dossier déclaré par
//!   `content_folders` (dossiers non récursifs, comme `Registry::build`), une fois par seconde. Elle
//!   couvre ce qu'aucun asset ne suit (`items/`, `rooms/`, `statuses/`, `patterns/`, `curses/`,
//!   `economy`, `perks`, `game.ron`) et l'édition à la main dans un éditeur de texte ;
//! - `alacod/reload_content` (`remote.rs`) appelle [`reload_registry_now`].
//!
//! Le registre n'est remplacé que si le lint est vert ; sinon l'ancien reste en place et les erreurs
//! sont journalisées. Le remplacement a lieu dans **tous** les états (lobby comme partie) : aucun
//! système de `GgrsSchedule` ne lit `Registry` ni `GameManifest` (les lecteurs sont les `OnEnter(GameLoading)`
//! qui redérivent leurs tables, le chargement de la carte, le démarrage de session et des écrans :
//! `docs/editeur-vocal.md` § 5). Une partie en cours garde donc ses tables dérivées jusqu'au prochain
//! `GameLoading` (`alacod/restart`). Ce système ne tourne jamais dans `GgrsSchedule`.
//!
//! Limite : `GlobalAsset` ne lit le registre qu'au `Startup` (handles des assets de personnages et
//! d'armes) ; un fichier de personnage **ajouté** n'est donc suivi comme asset qu'après relance.
//!
//! Idiome repris de `crates/game/src/ui/hud.rs` (`handle_hud_config_changes`) :
//! `MessageReader<AssetEvent<T>>` pour chacun des types de contenu déjà suivis comme assets Bevy.

use bevy::prelude::*;
use std::path::PathBuf;

use crate::character::config_asset::CharacterConfigAsset;
use crate::waves::assets::WaveConfigAsset;
use crate::weapon_assets::{MeleeWeaponsConfig, WeaponsConfig};
use content::{manifest::GameManifest, registry::Registry};

/// Dossier du jeu (ex. `games/zombies`, posé par `main.rs` depuis
/// `env!("CARGO_MANIFEST_DIR")`) : sert à relire `game.ron` et ses dossiers de contenu.
/// Absente dans les tests de scénario (`crates/scenario`), qui n'ont pas besoin du
/// rechargement à chaud : le système no-op si elle n'est pas posée.
#[derive(Resource, Clone)]
pub struct GameRoot(pub PathBuf);

pub struct ContentHotReloadPlugin;

impl Plugin for ContentHotReloadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, reload_registry_on_change);
    }
}

/// Résultat d'un rechargement : compteurs du registre **lu** (remplacé seulement si `errors` est vide).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReloadReport {
    pub reloaded: bool,
    pub characters: usize,
    pub weapons: usize,
    pub melee: usize,
    pub waves: usize,
    pub errors: Vec<String>,
}

/// Relit le contenu de `root` ; renvoie le registre et le manifeste neufs s'ils sont valides.
fn load_content(root: &std::path::Path) -> (Option<(Registry, GameManifest)>, ReloadReport) {
    match content::load_and_lint(root) {
        Ok((registry, manifest, errors)) => {
            let mut report = ReloadReport {
                reloaded: errors.is_empty(),
                characters: registry.characters.len(),
                weapons: registry.weapons.len(),
                melee: registry.melee_weapons.len(),
                waves: registry.waves.len(),
                errors: errors.iter().map(|e| e.to_string()).collect(),
            };
            if errors.is_empty() {
                (Some((registry, manifest)), report)
            } else {
                report.reloaded = false;
                (None, report)
            }
        }
        Err(e) => (
            None,
            ReloadReport {
                errors: vec![format!("game.ron invalide : {e}")],
                ..Default::default()
            },
        ),
    }
}

fn log_report(report: &ReloadReport) {
    if report.reloaded {
        info!(
            "contenu rechargé ({} personnages, {} armes, {} armes de corps à corps, {} vagues)",
            report.characters, report.weapons, report.melee, report.waves,
        );
    } else {
        for e in &report.errors {
            error!("contenu invalide, registre non rechargé : {e}");
        }
    }
}

/// Recharge le registre tout de suite (`alacod/reload_content`). `Err` si [`GameRoot`] est absente.
pub fn reload_registry_now(world: &mut World) -> Result<ReloadReport, String> {
    let root = world
        .get_resource::<GameRoot>()
        .map(|root| root.0.clone())
        .ok_or_else(|| "GameRoot absente : pas de dossier de jeu à relire".to_string())?;
    let (loaded, report) = load_content(&root);
    log_report(&report);
    if let Some((registry, manifest)) = loaded {
        world.insert_resource(registry);
        world.insert_resource(manifest);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(mut poll) = world.get_resource_mut::<ContentPoll>() {
        poll.fingerprint = None; // le prochain relevé sert de nouvelle base
    }
    Ok(report)
}

/// Empreinte des fichiers du contenu : ensemble trié de (chemin relatif à `assets/`, mtime, taille)
/// de `game.ron` et de chaque fichier ou dossier (non récursif) déclaré par `content_folders`.
/// Un fichier illisible compte comme absent. Vide si `game.ron` ne se lit pas.
#[cfg(not(target_arch = "wasm32"))]
pub fn content_fingerprint(root: &std::path::Path) -> Vec<(String, std::time::SystemTime, u64)> {
    use std::path::Path;
    let assets = GameManifest::assets_dir(root);
    let mut out = Vec::new();
    let mut push = |path: &Path, out: &mut Vec<(String, std::time::SystemTime, u64)>| {
        if let Ok(meta) = std::fs::metadata(path) {
            if meta.is_file() {
                let rel = path.strip_prefix(&assets).unwrap_or(path);
                out.push((
                    rel.to_string_lossy().replace('\\', "/"),
                    meta.modified().unwrap_or(std::time::UNIX_EPOCH),
                    meta.len(),
                ));
            }
        }
    };
    push(&GameManifest::manifest_path(root), &mut out);
    let Ok(manifest) = GameManifest::load(root) else {
        // `game.ron` en cours d'écriture ou invalide : l'empreinte de `game.ron` seule change déjà.
        out.sort();
        return out;
    };
    for decl in &manifest.content_folders {
        let path = assets.join(&decl.path);
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    push(&entry.path(), &mut out);
                }
            }
        } else {
            push(&path, &mut out);
        }
    }
    out.sort();
    out.dedup();
    out
}

/// État de la scrutation : dernière empreinte et instant du dernier relevé.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource, Default)]
struct ContentPoll {
    fingerprint: Option<Vec<(String, std::time::SystemTime, u64)>>,
    last: Option<std::time::Instant>,
}

/// Période de la scrutation. `Instant` et non `Time` : la partie du remote en pause fige `Time`, et
/// une édition faite pendant la pause doit se voir aussi.
#[cfg(not(target_arch = "wasm32"))]
const POLL_PERIOD: std::time::Duration = std::time::Duration::from_secs(1);

#[cfg(not(target_arch = "wasm32"))]
fn content_changed_on_disk(root: &std::path::Path, poll: &mut ContentPoll) -> bool {
    let now = std::time::Instant::now();
    if poll.last.is_some_and(|last| now.duration_since(last) < POLL_PERIOD) {
        return false;
    }
    poll.last = Some(now);
    let current = content_fingerprint(root);
    let changed = poll
        .fingerprint
        .as_ref()
        .is_some_and(|previous| *previous != current);
    poll.fingerprint = Some(current);
    changed
}

#[allow(clippy::too_many_arguments)]
fn reload_registry_on_change(
    game_root: Option<Res<GameRoot>>,
    mut characters: MessageReader<AssetEvent<CharacterConfigAsset>>,
    mut weapons: MessageReader<AssetEvent<WeaponsConfig>>,
    mut melee: MessageReader<AssetEvent<MeleeWeaponsConfig>>,
    mut waves: MessageReader<AssetEvent<WaveConfigAsset>>,
    mut commands: Commands,
    #[cfg(not(target_arch = "wasm32"))] mut poll: Local<ContentPoll>,
) {
    // `|` (pas `||`) : les quatre lecteurs doivent être vidés à chaque appel, même si un
    // des premiers a déjà trouvé un changement (sinon leurs curseurs prennent du retard).
    let events = characters.read().any(is_modified)
        | weapons.read().any(is_modified)
        | melee.read().any(is_modified)
        | waves.read().any(is_modified);

    let Some(game_root) = game_root else {
        return;
    };

    #[allow(unused_mut)]
    let mut changed = events;

    // Scrutation des fichiers (natif). Un changement vu par les événements d'assets renouvelle
    // aussi l'empreinte, pour que la même écriture ne recharge pas deux fois.
    #[cfg(not(target_arch = "wasm32"))]
    {
        let polled = content_changed_on_disk(&game_root.0, &mut poll);
        if changed && !polled {
            poll.fingerprint = Some(content_fingerprint(&game_root.0));
        }
        changed |= polled;
    }

    if !changed {
        return;
    }

    let (loaded, report) = load_content(&game_root.0);
    log_report(&report);
    if let Some((registry, manifest)) = loaded {
        commands.insert_resource(registry);
        commands.insert_resource(manifest);
    }
}

fn is_modified<T: Asset>(event: &AssetEvent<T>) -> bool {
    matches!(event, AssetEvent::Modified { .. })
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    fn temp_game(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("alacod-hot-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("assets/items")).unwrap();
        fs::write(
            dir.join("assets/game.ron"),
            r#"(name: "t", content_folders: [(path: "items", kind: "PowerUp"), (path: "camera.ron", kind: "Camera")], entry: (start_map: "x", default_seed: 1))"#,
        )
        .unwrap();
        fs::write(dir.join("assets/items/a.ron"), "()").unwrap();
        fs::write(dir.join("assets/camera.ron"), "()").unwrap();
        dir
    }

    #[test]
    fn fingerprint_changes_on_edit_and_new_file_and_ignores_undeclared() {
        let dir = temp_game("fp");
        let base = content_fingerprint(&dir);
        assert_eq!(base.len(), 3, "game.ron, items/a.ron, camera.ron : {base:?}");
        assert_eq!(base, content_fingerprint(&dir));

        // Fichier hors contenu déclaré : sans effet.
        fs::write(dir.join("assets/ignored.png"), "x").unwrap();
        assert_eq!(base, content_fingerprint(&dir));

        // Édition (taille change) d'un fichier d'un dossier déclaré.
        fs::write(dir.join("assets/items/a.ron"), "(1)").unwrap();
        let edited = content_fingerprint(&dir);
        assert_ne!(base, edited);

        // Nouveau fichier dans un dossier déclaré.
        fs::write(dir.join("assets/items/b.ron"), "()").unwrap();
        assert_eq!(content_fingerprint(&dir).len(), 4);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn poll_is_throttled_and_first_reading_is_a_baseline() {
        let dir = temp_game("poll");
        let mut poll = ContentPoll::default();
        assert!(!content_changed_on_disk(&dir, &mut poll), "premier relevé = base");
        fs::write(dir.join("assets/items/a.ron"), "(12345)").unwrap();
        assert!(
            !content_changed_on_disk(&dir, &mut poll),
            "moins d'une seconde : pas de relevé"
        );
        poll.last = Some(std::time::Instant::now() - Duration::from_secs(2));
        assert!(content_changed_on_disk(&dir, &mut poll));
        poll.last = Some(std::time::Instant::now() - Duration::from_secs(2));
        assert!(!content_changed_on_disk(&dir, &mut poll), "rien de neuf");
        let _ = fs::remove_dir_all(&dir);
    }
}
