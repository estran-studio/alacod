# Rapport — m1-v4-hud-throne (T1.18, voie V4, §32)

Branche partie de `m1-v4-ecran-mutation` (6af9763), `origin/main` mergé.

## État en cours

Code écrit, **pas encore compilé** (attente du « feu vert » d'orch). Reste : compilation, tests,
suite complète (aucune trace ne doit bouger), captures de `throne_progression`.

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
