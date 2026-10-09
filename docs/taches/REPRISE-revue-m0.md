# Reprise : revue humaine M0 + movement-feel (prompt à coller dans une nouvelle session)

Coller tout ce qui suit dans Claude Code, sur l'autre Mac de William, à la racine d'un clone de
`estran-studio/alacod`.

---

Tu reprends la revue humaine de M0 (tâche T3.3) avec William, commencée sur son MacBook le
2026-10-04 et interrompue pour changer de machine. Tout est sur la branche **`revue-m0-suite`**.

## 0. Mise en place

```bash
git fetch origin
git checkout -b revue-m0-suite origin/revue-m0-suite   # ou git checkout revue-m0-suite
git log --oneline -8
```
Prérequis : Rust nightly (`rust-toolchain.toml`). Première compilation à froid : 20 à 40 min.
Depuis `movement-feel`, le profil `dev` est optimisé (D30 fermée) : `make zombies` est jouable,
`--profile headless` reste le plus rapide.

## 1. Lire, dans cet ordre

1. `CLAUDE.md` : règles de déterminisme, obligatoires.
2. `docs/taches/README.md` : §1 hygiène (variante cloud/clone seul), §4 vérification, §5 preuve
   des traces, §7 rapport.
3. `docs/digests/revue-m0.md` : le carnet de la revue (R1–R9, S1–S2). C'est le fil conducteur.
4. `docs/taches/rapports/fix-revue-m0.md` : ce qui est fait et vérifié, avec les chiffres.
5. `docs/taches/rapports/movement-feel.md` : la nouvelle course et le nouveau dash (autre session).
6. `docs/jouer-a-deux.md` : la recette p2p.
7. `docs/taches/dettes.md` : dettes connues, à ne pas re-signaler.

## État au 2026-10-09 (deuxième Mac) — à lire en premier

- **Revue solo terminée côté William** : R4, R5 et R7 corrigés et validés en jeu, movement-feel
  validé (S3). Ses points à faire sont dans `docs/digests/revue-m0.md` § « Bilan de la revue
  solo » (S4–S8, R10) : ce sont des tâches M1 à ouvrir, pas des correctifs de cette branche.
- Nouveaux commits : `22776e5` **R7** (l'enregistrement n'était pas écrit à la fermeture de la
  fenêtre : `write_recording_on_exit` ordonné après `bevy::window::ExitSystems`), puis le carnet
  et `map_probe` qui liste les `Interactable`. Enregistrements dans `docs/captures/revue-m0/`.
- Méthode de revue à voix haute : OBS (écran + micro, source « Capture de fenêtre ») puis
  `/Applications/MacWhisper.app/Contents/MacOS/mw transcribe <vidéo> --language fr --format srt`,
  images avec `ffmpeg -ss <s> -i <vidéo> -frames:v 1`.
- **Reste à faire, dans l'ordre** : §3 point 4 (partie à deux, si William la veut encore), point 5
  (fusion de `origin/main`), point 6 (vérification complète : seule `make test_scenarios` a tourné
  sur `8ac2c0b` ; après R7, non relancée), points 7-8 (rapport, ligne `LIVRÉ`).
- **Orchestration** : le serveur où tournait la session orchestratrice (`upgrade-bevy-0-19-1`) ne
  répond plus. Tout ce qu'il faut pour la reprendre est dans le dépôt : `docs/taches.md` (journal,
  voies, ordre de merge), `docs/taches/README.md` (protocole), `docs/taches/PROMPT-KICKSTART.md`
  (prompt d'un agent de développement), `docs/taches/dettes.md`, les fiches `docs/taches/*.md` et
  les rapports `docs/taches/rapports/`. Les chemins `/home/wq/Project/bascanada/alacod_root/…` du
  kickstart sont ceux de la machine Linux : les adapter au clone utilisé.

## 2. Ce que contient `revue-m0-suite`

Base : `main` `26f1496`. Commits :
- `6156d22` **movement-feel** : course nerveuse (3000/3000 px/s²), dash à i-frames (64 px en
  8 frames, cooldown 24, 6 i-frames, appui gardé 8 frames), caméra indépendante du framerate,
  profil `dev` optimisé (D30). Vérifié par sa session : 184 scénarios, tests des crates, fuzz 8
  graines, lint, gen, bench strict. **Jamais joué par un humain.**
- `6d3fd8f` bless de movement-feel (182 traces rebénies + 2 neuves, preuve dans le message).
- `cd13787` (via la fusion) **R4** : après « Rejouer », les murs ne bloquaient plus rien
  (`CollisionGrids` non remise à zéro). Test `restart_keeps_walls_solid`, scénario
  `revue_murs_avant_poste`.
- `75a56d1` (via la fusion) **R5** : la touche R (rechargement) relançait la partie à tout
  moment. Test `r_ne_relance_que_sur_l_ecran_de_fin`.
- `4593fef` le carnet `docs/digests/revue-m0.md`.
- `83fcf9c` fusion, `8ac2c0b` bless de `revue_murs_avant_poste` sur l'état fusionné, puis le
  rapport provisoire et ce fichier.

Les branches d'origine `fix-revue-m0` et `movement-feel` sont aussi poussées.

## 3. Prochaines étapes, dans l'ordre

1. **Vérifier la machine** : `cargo build -p zombies --profile headless`, puis
   `make test_scenarios` (attendu : vert, 185 scénarios environ, aucune trace modifiée).
2. **Faire valider par William le ressenti de movement-feel** en jeu solo : course, dash,
   i-frames (« à la Gungeon » ; `dash_iframes: 0` donne l'esquive de Nuclear Throne). Noter
   chaque retour dans `docs/digests/revue-m0.md` (bug ou sensation ; les sensations ne se
   corrigent pas dans la revue, elles deviennent des tâches M1, sauf si William décide
   autrement).
   Lancer en enregistrant :
   ```bash
   ALACOD_RECORD=$PWD/tests/scenarios/revue_<sujet>.ron APP_VERSION=x \
     cargo run -p zombies --profile headless
   ```
   (`make record_session` compile en `dev` : R2.) Touches réelles (`control.rs`, R6) : WASD
   (pas les flèches, qui bougent la caméra, sauf flèche droite liée deux fois), **H** interagir,
   **C** dash, Tab arme suivante, R recharger, F mêlée, Shift gauche sprint, clic tirer.
3. **Questions ouvertes à William** (notées dans le carnet) :
   - R7 : la partie solo 2 n'a pas été enregistrée. Était-elle lancée avec `ALACOD_RECORD` ?
     Sinon, tester l'écriture à la fermeture de la fenêtre (`recording.rs`,
     `write_recording_on_exit`, `Last`).
   - R8 : graine fixe `default_seed` 123456 → même carte et mêmes power-ups à chaque partie
     (6 Double Points sur 7, reproduit par simulation du flux `loot`). Graine au hasard par
     partie (hors simulation, partagée par la session) sauf `--seed`, ou garder ?
   - S1 « le gameplay est vraiment mauvais », S2 « la carte ne fait aucun sens » : à préciser
     (zombies, tir, rythme, économie, feedback ; disposition, lisibilité, carte générée par
     `gen_map.py` hors dépôt), puis proposer des tâches M1. Ne pas trancher le design.
   - R2, R3, R6 : petits correctifs proposés, à faire seulement avec son accord.
4. **Partie à deux** (`docs/jouer-a-deux.md`) : signaling sur la machine Linux de William
   (`ws://100.64.0.6:3536`, il le démarre lui-même), les deux clones **au même commit**.
   Points à regarder : à terre/réanimation, power-ups partagés, fin de partie et retour au
   lobby (le restart p2p existe maintenant sur `main` par D14, pas encore dans cette branche),
   sons, fermeture, fps.
5. **Fusion de `origin/main`** (163 commits de plus que `26f1496` au 2026-10-05) : essai à
   blanc fait (`git merge-tree`) : **93 traces** en conflit et 8 fichiers :
   `crates/bots/src/decide.rs`, `crates/game/src/run_state.rs`, `docs/conventions.md`,
   `docs/taches/dettes.md`, `tests/scenarios/{bot_floors_three,throne_floor_1,
   throne_progression,throne_three_floors}.ron`.
   - `run_state.rs` : **prendre la version de `main`**, qui contient déjà le même correctif
     R4 (`f9f0d8e`) et le restart p2p (D14). Garder le test `restart_keeps_walls_solid`, le
     scénario `revue_murs_avant_poste` et R5 (`game_over.rs`), que `main` n'a pas.
   - Scénarios `.ron` en conflit : movement-feel les a recalés (positions, frames) et `main`
     les a modifiés aussi. Les recaler sur l'état fusionné (méthode `docs/conventions.md` §34).
   - Traces : conflit = les deux côtés ont re-béni. Prendre n'importe quelle version, puis
     preuve README §5 **contre le nouveau `main`** et bless sur l'état fusionné, justifié dans
     le commit.
   - Hors de portée si William préfère : laisser la fusion à l'orchestrateur et le dire.
6. **Vérification avant de livrer** (README §4) : `make test_scenarios`, tests des dix crates,
   `make lint`, `cargo fmt --all -- --check`, scripts (`check-forbidden.sh`,
   `check-rollback-registration.sh`), `make gen` (zombies, testbed, throne) sans modification ;
   p2p à deux clients si Docker est disponible, sinon « non vérifié ici ». Pas de bench sur un
   portable qui travaille.
7. **Rapport** : compléter `docs/taches/rapports/fix-revue-m0.md` (§7 : fait, vérifié avec
   chiffres réels, non fait, dettes). Ne jamais écrire « vérifié » pour ce qui n'a pas tourné.
8. **Livraison** : `git push origin revue-m0-suite`, puis donner à William la ligne
   `LIVRÉ revue-m0-suite <sha> : <une ligne>` ; il la relaie à l'orchestrateur, qui vérifie et
   merge. Ne pas merger dans `main`, ne pas toucher `docs/taches.md` ni la table de `dettes.md`
   (l'orchestrateur les tient ; movement-feel y a déjà ajouté D41 et marqué D30, à garder).

## 4. Règles (rappel)

Fixed (jamais f32 dans `GgrsSchedule`), `order_iter!`, `BTreeMap`, `despawn_rollback`,
`FrameEvents`, donnée dérivée réécrite chaque frame avant tout `continue` — et une donnée
dérivée mise en cache entre frames doit être remise à zéro avec la partie (leçon de R4).
Rien dans la simulation sans scénario. Un commit par correctif, message qui nomme la note.
Ne jamais bénir sans preuve (README §5, `scripts/trace-diff.py`).
