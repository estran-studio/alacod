# X-6h — Caméra fixe par pièce et portes fermées pendant le combat (brief du worker, 21 nov 2026)

Tu es le worker du lot **X-6h** de l'engine alacod. Coordinateur : le coordinateur alacod (il relit, fait relire et merge `main` ; toi jamais). Français partout.

## Pourquoi
Décision de William (2026-11-21, nuit) : ordre de l'engine validé après X-6g ; lot (3) : « caméra fixe par pièce et portes fermées pendant le combat, comme Isaac ». Jalon servi :
**D3 dim 6 déc** (cible jeu 3) de 1837. William joue et valide **à un joueur, en natif** ; la coop reste de première classe (scénarios à deux).
La fiche fait foi : `docs/taches/plan-1837-cli.md` § 6, ligne **X-6h** (contenu et preuve, écrits avant ton départ) ; § 3 « Décisions de William (2026-11-21) ».
Lots précédents à connaître : X-6f (porte de sortie vers le niveau suivant, `docs/conventions.md` § 128) et X-6g (intérieur par grammaire d'étage, § 129).

## Ce que tu construis (générique, noms neutres, rien de 1837 dans l'engine)
**Relève d'abord** : beaucoup existe (§ 35 salles typées verrouillées `locks: true`, `RoomStates` ; § 40 `CurrentRooms` et caméra `room_frame` / `frame_in_room` ; § 117 portes de carte et clé
d'équipe ; § 113 feu follet). Joue l'enchaînement réel du testbed (séquence `porte_sortie_interieur` : extérieur → porte → intérieur `floor:interieur`) avec des salles `locks: true`
et dis ce qui marche déjà et ce qui manque. Ensuite, la plus petite extension générique, en données :
1. **Caméra fixe par pièce** (présentation seule, jamais lue par la simulation) : la vue cadre la salle courante quelle que soit sa taille (mode en donnée de `camera.ron`, par ex. zoom
   ajusté pour que la salle tienne), glissement court en donnée d'une salle à l'autre ; hors salles typées (dehors), la caméra suit comme avant ; le défaut de `camera.ron` ne change rien
   aux jeux existants (isaac garde son `room_frame`).
2. **Portes de combat** : entrée dans une salle `locks: true` non vidée → ses portes se ferment (portes de carte de X-6a1 et portes générées comprises) ; salle vidée → elles s'ouvrent ; une
   porte à clé reste verrouillée tant que la clé manque ; une salle vidée ne se referme pas.
3. **Coop** : quand une salle se verrouille, un coéquipier resté dehors est amené à l'entrée (règle d'Isaac, en donnée, désactivable) **ou** la salle attend qu'il entre : choisis, justifie dans
   le rapport ; à terre / feu follet suivent la règle choisie. Caméra : en local, la salle du joueur suivi ; en ligne, chacun la salle de son joueur (`online_follow`).

**Hors périmètre** (`propositions.md`, ne pas faire) : carte dessinée (`layout: Drawn`), image propre de porte, trois thèmes de musique (lot (5)), sergent, lutin, `p2p-stress.sh` externe.

## Preuve attendue (rien ne merge sans elle)
(a) Relevé (fichier:ligne) avant code ; trois lignes « Ce qui existait déjà / Ce qui manquait / Ce qui est ajouté » en tête du rapport.
(b) Scénarios neutres du testbed, traces neuves seulement, **à un joueur** : entrée dans une salle de combat → portes fermées (`MapDoor`, `RoomState`), ennemis tués → ouvertes ; porte à clé
  fermée sans clé après le combat ; salle déjà vidée qui ne se referme pas ; **à deux joueurs** : la règle coop choisie, coéquipier à terre ou en feu follet. `// À regarder : …` en tête.
(c) Test pur de la caméra (salle plus petite, égale, plus grande que la vue ; glissement borné) ; une attente de présentation si utile (comme `HudText`, hors trace).
(d) **Capture de 20 à 60 s du vrai jeu** natif release du testbed, fenêtre sur ce Mac, jouée par toi (`alacod remote` : `input`, `step`, `screenshot`) : couloir → salle de combat qui se ferme →
  combat → portes rouvertes → salle suivante, caméra fixe par pièce. Sous `docs/taches/rapports/preuves/x-6h/` (vidéo ≤ 10 Mo + 3 images). Supprime `target/release` après.
(e) Tableau des traces avant/après des familles touchées (isaac et gungeon à salles verrouillées, `room_*`, `key_door*`, `sortie_porte_*`, `interieur_*`, `wisp_*`) par
  `make test_scenarios SCENARIO=a,b,…` : existantes inchangées sauf BLESS justifié d'une ligne. Si tu ajoutes un système ou un type rollback : variante **neutre**, test `scenario/schedule_order`
  vert ou changement expliqué.
(f) Contrôle rapide **découpé** vert sur ta tête poussée. Le coordinateur fait relire ensuite.

## Machine : le MacBook personnel de William (règles dures)
- 8 cœurs, 16 Go de RAM partagés avec sa partie. Un seul `cargo` à la fois, jamais en arrière-plan. Préfixe : `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1`.
- **Jamais de profil debug** : `--profile headless` pour tout test/vérification (le Makefile le fait) ; `--release` seulement pour la capture.
- Disque : ~45 Go libres, **plancher 30**. `df -h /System/Volumes/Data` avant tout gros build ; sous 40 Go, `scripts/target-sweep.sh target` ; sous 30 Go, `ask`.
  La capture release et le contrôle rapide l'un après l'autre, jamais ensemble.
- Contrôle rapide : jamais `make check_rapide` d'un bloc. Depuis ton worktree :
  `LOGS=$HOME/orca/runs/alacod/logs bash docs/taches/specs-coordinateur/scripts/p-1d-check-rapide-decoupe.sh`, puis `scripts/target-sweep.sh target` ;
  journal copié en `$HOME/orca/runs/alacod/logs/x-6g-check-rapide-<sha>.log`.
- Suite complète jamais sur le Mac (la nuit du VPS la joue après le merge).
- Si `GITHUB_TOKEN` est dans ton environnement : `env -u GITHUB_TOKEN git push …`.
- Ne touche jamais `/Users/wq/Project/bascanada/alacod` ni `/Users/wq/Project/bascanada/1837` (lecture par `git show` seulement). `1867_lore` : lecture seule.
- `pkill`/`killall` par motif interdits ; aucun processus laissé à la fin. Si tu modifies `games/testbed/assets/game.ron` ou `camera.ron` pour la capture, rétablis-les sans les commiter.

## Règles du dépôt
`CLAUDE.md` (checklist GGRS ; **frontière simulation/présentation** : la caméra est dans `PresentationPlugin`, jamais lue par `GgrsSchedule` ; les visuels se dérivent de l'état).
Conventions : § **130** de `docs/conventions.md` ; dettes **D228–D229**. Rapport `docs/taches/rapports/x-6h-camera-portes-combat.md` (par point de la preuve : issue et chemin ;
dernières lignes du contrôle rapide ; ce qui reste). Ligne `LIVRÉ wquintal/x-6h-camera-portes-combat : …` à la fin de `docs/taches.md`. Mets à jour la section caméra de `CLAUDE.md` (une ligne) si le contrat change.

## Communication
- Branche `wquintal/x-6h-camera-portes-combat`, poussée à chaque point de contrôle (avance rapide, jamais `--force`).
- Carte : `orca worktree set --worktree active --comment "<état>" --json` à chaque étape.
- `status` au coordinateur à chaque étape franchie (relevé, scénarios, capture) ; **aperçu à mi-parcours obligatoire si le lot dépasse deux jours**.
- Question bloquante : `ask` court avec ton choix par défaut, et avance en attendant. Aucune question de diagnostic à William. 429 / quota : `status` immédiat.
- Prêt : `status` « X-6h prêt <sha> » avec le chemin du journal. Fin : rapport commité, branche poussée, **un seul** `worker_done`.
