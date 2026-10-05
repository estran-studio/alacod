# m1-dettes-doc-lot-2 — suites de la relecture des conventions

## État en cours

- Fait : les trois décisions d'orch appliquées, six commentaires périmés corrigés, en-tête de
  `powerups.ron` (valeurs inchangées), commentaire de la cible `make gen`, deux « §8 » de
  `docs/taches/README.md`. Branche depuis `e75ec2f` (m1-relecture-conventions), même target.
- **`cargo check` non lancé, pas de feu vert** (machine prise par les 200 graines et la
  vérification groupée) : à lancer par orch, `cargo check -p game -p run -p sim_core --profile
  headless`. `make lint GAME=throne` non lancé non plus (seuls des commentaires de
  `powerups.ron` changent). `bash -n scripts/nightly.sh` : OK.
- Contrôle : `git diff -U0` des `.rs`, `.ron` et du `Makefile` ne contient que des lignes de
  commentaire (`//`, `///`, `//!`, `#`).

## Fichiers

| Fichier | Changement |
|---|---|
| `docs/conventions.md` §6 | « Vidéos MP4 » : chemin `nightly/<commit>/videos/` dans l'artefact (décision 3) |
| `docs/conventions.md` §9 | exception `variant_health` documentée pour de bon, avec la raison (décision 1) |
| `docs/conventions.md` Notes essentielles | toute voie peut bénir avec la preuve du §10, bénie par l'orchestrateur ; V1 le plus souvent (décision 2) |
| `crates/game/src/character/variant.rs` | `//` au point d'appel de `resolve` (décision 1) |
| `scripts/nightly.sh` | copie des MP4 dans `nightly/<commit>/videos/` (décision 3, diff ci-dessous) |
| `.github/workflows/nightly.yaml` | inchangé : l'artefact envoie déjà `nightly/${{ env.COMMIT }}/` entier |
| `crates/game/src/character/health/mod.rs` | six émetteurs de `DamageEvent` (et le traducteur de charge) |
| `crates/game/src/run_state.rs` | restart p2p livré (D14, §33), seul `--allumette` redirigé |
| `crates/run/src/run.rs` | `Floors` livré (T1.8, §17), `Campaign` en M2 (deux commentaires) |
| `crates/sim_core/src/kinds.rs` | chemin `crates/combat/src/weapons/mod.rs` |
| `games/throne/assets/items/powerups.ron` | en-tête : un ennemi sur quatre (`drop_chance` 0,25), poids 36/12/12/12/24 et 25/25, total 146 ; aucune valeur changée |
| `Makefile` | commentaire de `gen` : armes **et** ennemis à `test:` (v1, T1.13). Correction du rapport précédent : la recette lance bien `alacod-gen` (pas `map_generation`) ; seul le périmètre décrit était périmé |
| `docs/taches/README.md` | l. 82 et 145 : §8 → §10 |

## Diff de `scripts/nightly.sh`

Placée après le pas « Video generation done » (D) : les vidéos sont écrites par
`scripts/scenario-video` dans `<racine>/target/videos/<commit court>[-dirty]/`, hors de
`NIGHTLY_DIR` (`$CARGO_TARGET_DIR/nightly/<commit>`). Avec `set -e`, `cp … && …` n'arrête pas le
script si une copie échoue, et l'absence de vidéo laisse un dossier vide (copie : 0).

```diff
diff --git a/scripts/nightly.sh b/scripts/nightly.sh
index e9f43ee..6395dc1 100755
--- a/scripts/nightly.sh
+++ b/scripts/nightly.sh
@@ -342,6 +342,22 @@ END_TIME=$(date +%s)
 DURATION=$((END_TIME - START_TIME))
 log_step "Video generation done (${DURATION}s)"
 
+# Copie des MP4 dans l'artefact (`nightly/<commit>/videos/`, envoyé en entier par
+# `.github/workflows/nightly.yaml`) : `make videos` écrit dans `target/videos/<commit>/` à la
+# racine du dépôt (`scripts/scenario-video`, commit court, suffixe `-dirty` possible), hors de
+# `NIGHTLY_DIR`. `*.vues.mp4` est couvert par le motif ; aucune vidéo n'est pas une erreur.
+VIDEOS_SRC_ROOT="$(git rev-parse --show-toplevel)/target/videos"
+mkdir -p "${NIGHTLY_DIR}/videos"
+VIDEOS_COPIED=0
+for VIDEOS_SRC in "${VIDEOS_SRC_ROOT}/${COMMIT}" "${VIDEOS_SRC_ROOT}/${COMMIT}-dirty"; do
+    [ -d "${VIDEOS_SRC}" ] || continue
+    for MP4 in "${VIDEOS_SRC}"/*.mp4; do
+        [ -f "${MP4}" ] || continue
+        cp "${MP4}" "${NIGHTLY_DIR}/videos/" && VIDEOS_COPIED=$((VIDEOS_COPIED + 1))
+    done
+done
+log_step "Videos copied to artifact: ${VIDEOS_COPIED}"
+
 # ============================================================================
 # STEP E: REVIEW PAGE
 # ============================================================================
```
