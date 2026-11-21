# X-6g — Intérieur par graine (brief du worker, 21 nov 2026)

Tu es le worker du lot **X-6g** de l'engine alacod. Coordinateur : le coordinateur alacod (il relit, fait relire et merge `main` ; toi jamais). Français partout.

## Pourquoi
Décision de William (2026-11-21), lot (2) de l'engine : « génération de l'intérieur par graine avec `BasicMapGeneration` : hall au centre à trois sorties ; réunion,
cuisine, réserve, salle de la clé et salle du chef garanties ; clé et chef dans deux branches différentes ; une graine par défaut, rechargée à chaque relance (pas de
sauvegarde pour l'instant) ». Jalon servi : **D3 dim 6 déc** (cible mer 2) de 1837. Le lot (1) X-6f (changement de niveau par une porte, `docs/conventions.md` § 128)
est mergé : l'intérieur s'enchaîne après l'extérieur par sa porte.
La fiche fait foi : `docs/taches/plan-1837-cli.md` § 6, ligne **X-6g** (contenu et preuve, écrits avant ton départ) ; § 3 « Décisions de William (2026-11-07) » et
« (2026-11-21) ».

## Ce que tu construis (générique, noms neutres, rien de 1837 dans l'engine)
Extensions **opt-in** de la génération (`BasicMapGeneration`, `crates/map/src/generation/imp/basic.rs`, un niveau LDtk = une salle, prises `LevelConnection`) et de la
grammaire d'étage du § 40 (`FloorMapGeneration`, `crates/map/src/generation/floor.rs`, `imp/floor.rs` : départ au centre, salles requises, boss en cul-de-sac, nouvelles
tentatives). **Relève d'abord ce qui sert** et choisis le point d'extension le plus petit ; pas le planificateur de grille d'isaac (§ 70, `grid.rs`).
1. **Salle centrale** posée au centre de l'emprise, jamais tirée au bord, avec un **nombre exact de sorties** en donnée (3 pour 1837).
2. **Types garantis** : chaque type requis présent exactement le nombre demandé ; plan refusé et retiré sinon (jamais une carte incomplète), échec explicite après N tentatives.
3. **Branches distinctes** : deux types désignés en donnée au bout de deux branches différentes partant de la salle centrale ; la porte de l'un verrouillable par la clé
   portée/posée dans l'autre (X-6a1, en donnée).
4. **Graine** : une graine par défaut du jeu (donnée, ex. `entry.default_seed` s'il suffit), la même à chaque relance, pas de sauvegarde ; une graine explicite (`--seed`,
   scénario) la remplace. Vérifie le comportement actuel de la relance avant d'écrire quoi que ce soit.
5. Défauts relevés par 1837 sur `36ab2386` (branche 1837 `wquintal/cartes-prologue`, `docs/tranche.md` lignes ~400-410) : une seule salle du chef en six graines
   (5 sur 6 le 10 nov) ; une graine (5) à 4 salles, son hall tiré au bord haut, prises du nord murées faute de place (cause probable : `get_spawning_room` tire la salle
   de départ dans `[-max_width, max_width - largeur]`, `world::MapSettings`). Établis la cause et couvre-la par un test.
Le **défaut** de chaque extension ne change rien : cartes `Basic` des clones et étages `floor:` existants identiques au bit près (RNG de génération compris).

Lecture de 1837 permise sans écrire dans son checkout : `git -C /Users/wq/Project/bascanada/1837 show origin/wquintal/cartes-prologue:assets/maps/loge_interieur.ldtk > /tmp/loge_interieur.ldtk`
(idem pour ses `assets/rooms/` et `game.ron`), puis `alacod-map` dessus. **Aucun contenu, nom ni valeur de 1837 dans l'engine** : la carte du testbed est neutre.

**Hors périmètre** (`propositions.md`, ne pas faire) : carte dessinée posée à la position LDtk (`layout: Drawn`), image propre de porte, caméra fixe par pièce et portes
fermées pendant le combat, `p2p-stress.sh` externe. Toute autre idée : une ligne dans `propositions.md`, et tu continues.

## Preuve attendue (rien ne merge sans elle)
(a) Relevé (fichier:ligne) en tête du rapport, **avant tout code** : `BasicMapGeneration`, § 40, relance et graine, cause des deux défauts de 1837.
(b) **Campagne sur 200 graines** d'une carte neutre du testbed (`alacod-map` ou un test ignoré) : 200/200 avec salle centrale au centre à 3 sorties, chaque type garanti
  exactement une fois, les deux types désignés dans deux branches distinctes ; nombre de nouvelles tentatives (moyenne, max) et temps de génération (moyen, max).
  Même campagne en lecture sur `loge_interieur.ldtk` de 1837 (résultat dans le rapport ; si ses gabarits ne permettent pas la garantie, dis précisément quoi changer
  dans **son** contenu : c'est pour le coordinateur 1837).
(c) Scénarios neutres du testbed, traces neuves seulement, **à un et à deux joueurs** : extérieur → porte (X-6f) → intérieur généré ; graine par défaut identique après relance ;
  graine explicite différente ; porte verrouillée refusée sans la clé. Chaque scénario commence par `// À regarder : …`.
(d) **Capture de 20 à 60 s du vrai jeu** natif release du testbed, fenêtre sur ce Mac, jouée par toi (`alacod remote` : `input`, `step`, `screenshot`), vidéo : arrivée dans la
  salle centrale, deux branches visitées ; plus le rendu PNG de six graines. Sous `docs/taches/rapports/preuves/x-6g/` (vidéo ≤ 10 Mo). Supprime `target/release` après.
(e) Tableau des traces avant/après des familles touchées (cartes `Basic` : testbed, zombies, throne ; étages `floor:` : gungeon, isaac ; `sortie_porte_*`), par
  `make test_scenarios SCENARIO=a,b,…` : traces existantes inchangées sauf BLESS justifié d'une ligne. Si la simulation change (système, type rollback) : le test
  `scenario/schedule_order` doit rester vert, et dis-le.
(f) Contrôle rapide **découpé** vert sur ta tête poussée. Le coordinateur fait relire ensuite.

## Machine : le MacBook personnel de William (règles dures)
- 8 cœurs, 16 Go de RAM partagés avec sa partie. Un seul `cargo` à la fois, jamais en arrière-plan. Préfixe : `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1`.
- **Jamais de profil debug** : `--profile headless` pour tout test/vérification (le Makefile le fait) ; `--release` seulement pour la capture.
- Disque : ~44 Go libres, **plancher 30**. `df -h /System/Volumes/Data` avant tout gros build ; sous 40 Go, `scripts/target-sweep.sh target` ; sous 30 Go, `ask`.
  La capture release et le contrôle rapide l'un après l'autre, jamais ensemble.
- Contrôle rapide : jamais `make check_rapide` d'un bloc. Depuis ton worktree :
  `LOGS=$HOME/orca/runs/alacod/logs bash docs/taches/specs-coordinateur/scripts/p-1d-check-rapide-decoupe.sh`, puis `scripts/target-sweep.sh target` ;
  journal copié en `$HOME/orca/runs/alacod/logs/x-6g-check-rapide-<sha>.log`.
- Suite complète jamais sur le Mac (la nuit du VPS la joue après le merge).
- Si `GITHUB_TOKEN` est dans ton environnement : `env -u GITHUB_TOKEN git push …`.
- Ne touche jamais `/Users/wq/Project/bascanada/alacod` ni `/Users/wq/Project/bascanada/1837` (lecture par `git show` seulement). `1867_lore` : lecture seule.
- `pkill`/`killall` par motif interdits ; aucun processus laissé à la fin. Si tu modifies `games/testbed/assets/game.ron` pour la capture, rétablis-le sans le commiter.

## Règles du dépôt
`CLAUDE.md` (checklist GGRS : `BTreeMap`, RNG de génération consommé dans un ordre déterministe, numérotation `GgrsNetId` par clé de contenu, frontière simulation/présentation).
Conventions : § **129** de `docs/conventions.md` ; dettes **D226–D227**. Rapport `docs/taches/rapports/x-6g-interieur-graine.md` (par point de la preuve : issue et chemin ;
dernières lignes du contrôle rapide ; ce qui reste). Ligne `LIVRÉ wquintal/x-6g-interieur-graine : …` à la fin de `docs/taches.md`. Mets à jour § 1 / § 40 si tu étends leur contrat.

## Communication
- Branche `wquintal/x-6g-interieur-graine`, poussée à chaque point de contrôle (avance rapide, jamais `--force`).
- Carte : `orca worktree set --worktree active --comment "<état>" --json` à chaque étape.
- `status` au coordinateur à chaque étape franchie (relevé, campagne, scénarios, capture) ; **aperçu à mi-parcours obligatoire si le lot dépasse deux jours** (campagne + une capture).
- Question bloquante : `ask` court avec ton choix par défaut, et avance en attendant. Aucune question de diagnostic à William. 429 / quota : `status` immédiat.
- Prêt : `status` « X-6g prêt <sha> » avec le chemin du journal. Fin : rapport commité, branche poussée, **un seul** `worker_done`.
