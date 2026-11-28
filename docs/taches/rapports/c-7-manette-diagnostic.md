# C-7 — Manette : diagnostic et aide

**Prêt pour la partie de William** (pas livré avant son verdict). Code testé : `49e0f869` (branche `wquintal/c-7-manette-diagnostic`).
- Cause du verdict (« ni tir ni visée ») : Parsec tenait la manette. Le lot ajoute un journal de diagnostic, un avertissement nommé et l'aide ; aucune liaison ni aucun seuil changé.
- À faire avec William : preuve en direct (PC Linux, sans Parsec) et dans Chrome, commandes ci-dessous ; je ne les ai pas simulées.

- **Existait** : liaisons leafwing de la manette (`control.rs`), lecture dans `input_devices.rs` ; aucun journal de ce que la manette transmet.
- **Manquait** : voir ce que le jeu reçoit, et un signal quand une manette est tenue ailleurs.
- **Ajouté** : `character/gamepad_diagnostics.rs` (§ 134) : `ALACOD_INPUT_DEBUG=1` (connexion : nom, `fabricant:produit`, motif de GUID, source du mapping ; puis axes et boutons bruts au-delà de 0,05 de variation) ; `warn!` une fois par manette muette (1 800 frames de jeu avec un joueur local) ; `templates/starter/README.md` ; ligne `CLAUDE.md`.

## Preuve (VPS ovh, journaux dans `~/orca/runs/alacod/logs/` sur ovh)
- Tests `Pad` (`c-7-pad.log`) : muette ⇒ avertie une seule fois ; active ⇒ jamais ; sans joueur local ⇒ jamais ; motif de GUID de `045e:0b13` ⇒ `????00005e040000130b0000????0000` ; les tests de liaisons d'avant restent verts (10/10).
- `gestes_entree_reelle` (`c-7-gestes.log`) : 4/4 verts.
- Contrôle rapide découpé (`c-7-check-rapide-49e0f869.log`) : `make` sans `test_crates` code 0 ; `test_crates` découpé : 61 binaires, 0 échec, code 0 (1 966 s, total 2 879 s). Format corrigé en cours de route (`cargo fmt`, première passe arrêtée à 3 s, `check-rapide-decoupe-762489ec.log`).
- Aucune `.trace` modifiée (`git diff --stat` : aucune), aucun BLESS, aucun système de `GgrsSchedule` : `schedule_order_diff` non joué.

## Preuve en direct à faire avec William

**Natif (PC Linux, sans `parsecd`, sans Steam Input)** :
```
ALACOD_INPUT_DEBUG=1 alacod sandbox <jeu> 2>&1 | grep -i manette
```
(ou `ALACOD_INPUT_DEBUG=1 cargo run --features native -- --sandbox`). À lire dans le journal :
1. À la connexion : `manette connectée : nom="Xbox Wireless Controller" id=045e:0b13 guid=????00005e040000130b0000????0000 mapping=base SDL embarquée par gilrs …` (le bus et la version sont `????` : bevy ne les transmet pas ; le GUID complet attendu est `050000005e040000130b000015050000`).
2. Stick droit poussé : lignes `axe RightStickX = …` et `axe RightStickY = …` (SDL : `ABS_Z`/`ABS_RZ`). Absentes ⇒ le jeu ne reçoit pas la visée.
3. RT enfoncée : `bouton RightTrigger2 = …` (SDL : `ABS_GAS`) ; le tir part au-delà de 0,5.
4. Avec Parsec lancé (contre-épreuve) : aucune ligne d'axe ni de bouton, puis, après ~30 s de jeu, le `WARN manette "…" connectée mais muette … (Parsec, Steam Input …)`.

**Chrome (API Gamepad, build web)** : `alacod build --web --serve`, ouvrir la page, appuyer sur un bouton de la manette, puis dans la console de Chrome :
```js
const p = navigator.getGamepads()[0]; [p.id, p.mapping, p.buttons[7].value, p.axes[2], p.axes[3]]
```
Lire : `mapping` = `"standard"`, `buttons[7]` (RT) monte à 1 quand on appuie, `axes[2]` et `axes[3]` bougent avec le stick droit ; relancer la ligne pendant l'appui. Si ces valeurs bougent mais que le jeu ne tire pas ou ne vise pas, la cause est dans la liaison ; sinon, un autre onglet ou programme tient la manette. Le journal `ALACOD_INPUT_DEBUG` n'existe pas côté web (pas d'environnement) : la console de Chrome fait foi.

## Réserves
- Une manette posée que personne ne touche déclenche l'avertissement après ~30 s de jeu (indiscernable d'une manette tenue ailleurs) : le message le dit.
- La source du mapping est « variable `SDL_GAMECONTROLLERCONFIG` » ou « base embarquée par gilrs » ; gilrs ne dit pas quelle ligne il a retenue.
