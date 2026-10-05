# Rapport — m1-v4-hud-throne (T1.18, voie V4, §32)

Branche partie de `m1-v4-ecran-mutation` (6af9763), `origin/main` mergé.

## État

Livrée : suite headless verte, aucune trace déplacée, captures faites ; `origin/main` (26f1496,
D40) remergé et revérifié.

## Fait

- `content::ui::HUD_SOURCES` (liste fermée) + `HudFileSchema` ; lint `UnknownKind` « hud :
  source inconnue » ; fixture `hud_unknown_source` ; test : les `hud.ron` des trois jeux n'ont
  que des sources connues. Le `warn!` du jeu lit la même liste.
- `game::ui::hud_model` : `hud_values` (pure), `player_source_text` (sources du joueur, dont
  `rads`, `level`, `ammo_by_type`, `statuses`, `floor`), `HudSnapshot` (`HudModelPlugin`, aussi en
  headless), tests unitaires ; `ui::hud` lit le snapshot (barres `health` et `rads`).
- Attente `HudText { source, contains, at_frame }` (runner : lecture de `HudSnapshot`) ; test
  `hud_text_sur_throne_progression` (scénario inchangé) ; ligne dans la liste des attentes de
  `CLAUDE.md`.
- `games/throne/assets/ui/hud.ron` : étage (au lieu de la vague), barre et texte de rads,
  niveau, statuts, munitions par type.
- `docs/conventions.md` §32 + une ligne au §15 ; `CLAUDE.md` § HUD (sources).

## Vérifié

`CARGO_BUILD_JOBS=2`, profil `headless`, `origin/main` 5cad4b3 mergé :

- `cargo test -p game ui::` : 23 passés (dont `hud_model` : rads vers le seuil, niveau 0 et
  dernier niveau, rien sans progression ni `Floors`, munitions `Custom`, statuts, étage,
  snapshot) ; `cargo test -p content` : vert (dont `hud_unknown_source` et les `hud.ron` des
  trois jeux).
- `hud_text_sur_throne_progression` : vert — `Étage 1`, `balles`, `Niv. 0`, `rads` à f60,
  `Niv. 1` à f540, `Niv. 2` à f1580, `Étage 3` à f1912 ; échecs attendus (`Niv. 2` à f540,
  `brulure` à f60) bien signalés.
- `make test_scenarios` : vert, **aucune trace déplacée**, aucun bless.
- Tests des crates, `make lint` (trois jeux), `cargo fmt --check` (après `cargo fmt`),
  `check-forbidden`, `check-rollback-registration`, `cargo check -p throne`, exemples : verts.
- `make gen` testbed et zombies : vert, rien de régénéré ; `GAME=throne` : panique connue sur
  `arsenal`, sans lien.

Premier essai : une erreur de compilation (`HudPlayerState` privé dans une signature de système
ordonnée depuis `ui::hud`), corrigée (`pub(crate)`).

Après merge d'`origin/main` 26f1496 (D40 : `RefillAmmoOf`, `throne_progression` rebénie) : phrase
`RefillAmmoOf` ajoutée à `describe` (« munitions lames rechargées », même texte que la correction
d'orch sur T1.16) ; `effects`, `game ui::`, `content`, `hud_text_sur_throne_progression`,
`make test_scenarios` (aucune trace déplacée), `make lint`, `cargo fmt --check` : verts ;
`make gen GAME=throne` désormais **vert**.

## Captures

`play_scenario --capture` (`--profile headless --features render`, toutes les 4 frames), avant le
merge de D40 (le HUD ne dépend pas du butin), images dans `m1-v4-hud-throne/` :

- `throne_progression_f540.png` : niveau 1 (« Niv. 1 », « 3 / 8 rads », barre de rads vide :
  repart du seuil du niveau 1), « Étage 1 » en haut, « balles 396 / lames 48 » en bas à droite ;
  l'écran de mutation (T1.16) s'ouvre aussi pour le bot.
- `f1140` : choix pris d'office (Sang-froid), écran fermé.
- `f1400` : « 6 / 8 rads », barre de rads aux trois cinquièmes, « Étage 2 ».
- `f1580` : niveau 2 ; `f1912` : « Étage 3 » pendant le fondu de transition (T1.16).

Corrigé après la première série : « Niv. » chevauchait la ligne de debug de la frame (`Wave 0 |
--- | …`, `FrameDebugUIPlugin`) ; déplacé à droite du texte de vie, recapturé (les assets sont lus
au lancement : pas de recompilation).

## À valider à l'écran (William)

- Disposition du coin bas-gauche (vie, niveau, rads, statuts) ; aucun statut n'apparaît dans
  `throne_progression` (le joueur n'en reçoit pas) : la source `statuses` n'est vue que par les
  tests.
- La ligne de debug de la frame (« Wave 0 | … ») et le texte de caméra restent affichés en
  throne : hors de cette tâche.
