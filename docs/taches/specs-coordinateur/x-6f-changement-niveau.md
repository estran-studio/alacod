# X-6f — Changement de niveau par une porte (brief du worker, 21 nov 2026)

Tu es le worker du lot **X-6f** de l'engine alacod. Coordinateur : le coordinateur alacod (il relit, fait relire et merge `main` ; toi jamais).
Français partout (code commenté, rapport, messages).

## Pourquoi
Décision de William (2026-11-21), lot (1) de l'engine : « passer la porte principale de l'extérieur, clé en main, charge le niveau intérieur ». Jalon servi :
**D2 dim 29 nov** (cible jeu 26) de 1837 : « la porte s'ouvre » et mène à l'intérieur. Le lot suivant (X-6g, intérieur par graine) attend ton merge.
La fiche fait foi : `docs/taches/plan-1837-cli.md` § 6, ligne **X-6f** (contenu et preuve, écrits avant ton départ). Lis aussi § 3 « Décisions de William (2026-11-21) ».

## Ce que tu construis (générique, noms neutres, rien de 1837 dans l'engine)
Partir de l'existant, ne pas écrire un second système :
- porte de carte à états et clé d'équipe : `docs/conventions.md` § 117 (X-6a1, `lock_key`, `lock_consume`, `TeamKeys`, `KeyPickup`) ;
- séquence `Floors` et passage de niveau : § 17 (`floor_transition_system`, `FloorState`, mondes préchargés, numérotation continue) ;
- étages assemblés dans une séquence : § 40 « Étapes 2 » (graine dérivée par emplacement).

Une porte de carte peut être marquée **en donnée** (champ de porte LDtk, nom neutre à choisir) comme **sortie vers le niveau suivant**. Ouverte (clé d'équipe en main,
consommée ou non selon la donnée), un joueur debout qui la franchit fait passer **toute l'équipe** au niveau suivant (règles du passage du § 17), sans attendre que le
niveau soit vide. Un niveau (ou une séquence) qui sort par porte n'ouvre pas le portail « niveau vidé » (donnée ; **le défaut ne change rien** aux séquences existantes).
Le niveau suivant peut être une carte assemblée par graine. Relance = retour au premier niveau, même graine. Lint : porte de sortie sur le dernier niveau ; niveau à sortie
par porte sans porte de sortie. Prompt de la porte de sortie ouverte (« Entrer » ou équivalent, présentation).
Coop : un joueur franchit, les deux arrivent ; un coéquipier à terre ou en feu follet (M-5, § 113) arrive dans son état.

**Hors périmètre** (déjà dans `propositions.md`, ne pas faire) : carte dessinée posée à la position LDtk (`layout: Drawn`), image propre d'une porte.
Toute autre idée : une ligne dans `propositions.md` (racine), et tu continues.

## Preuve attendue (rien ne merge sans elle)
(a) Relevé (fichier:ligne) du passage de niveau, de la porte de carte et de `TeamKeys` **avant tout code**, en tête du rapport.
(b) Scénarios **neutres du testbed** (`games/testbed`, `tests/scenarios/`), traces neuves seulement :
  - **à un joueur** : clé ramassée → porte ouverte → franchie → `FloorIndex(1)`, joueur sur le départ du niveau suivant, santé, arme et munitions gardées ;
    témoin sans clé (porte refusée, `FloorIndex(0)`) ; témoin niveau vidé sans franchir (aucun passage) ; niveau suivant assemblé par graine ;
  - **à deux joueurs** : l'un franchit, les deux arrivent ; coéquipier à terre ou en feu follet suit dans son état ;
  - relance : retour au niveau 0.
  Chaque scénario commence par `// À regarder : …`. Les mettre dans le jeu court étiqueté `rapide` si c'est bon marché (§ 123).
(c) **Capture de 20 à 60 s du vrai jeu** : binaire natif **release** du testbed, fenêtre sur ce Mac (vrai GPU), joué par toi (`alacod remote` : `input`, `step`,
  `screenshot` ; ou entrées scriptées `ALACOD_INPUT=script:`), enregistré en vidéo (ffmpeg sur les captures, ou `screencapture -v`) : la clé, la porte, l'arrivée au niveau
  suivant. Une vidéo de scénario (`make videos`) peut s'ajouter, pas remplacer. Fichier sous `docs/taches/rapports/preuves/x-6f/` (vidéo compressée ≤ 10 Mo, plus 3 images).
(d) **Tableau des traces avant/après** des familles touchées (`Floors`/étages : `floors_*`, `etage*`, `generated/*` à étages ; `key_door*`, portes de carte ; `wisp_*`) joués
  sur `ca3d7e28` (avant) et sur ta tête (après) par `make test_scenarios SCENARIO=a,b,…` : traces existantes inchangées sauf BLESS justifié d'une ligne chacun.
  `make schedule_order_diff BASE=ca3d7e28` si la simulation change (ordre des systèmes et types rollback).
(e) Contrôle rapide **découpé** vert sur ta tête poussée (voir plus bas), journal complet.
Le coordinateur fait ensuite relire par un relecteur ; corrige ce qu'il demande.

## Machine : le MacBook personnel de William (règles dures)
- 8 cœurs, 16 Go de RAM, **partagés avec sa partie**. Un seul `cargo` à la fois, jamais en arrière-plan. Préfixe : `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1`.
- **Jamais de profil debug** : toute commande cargo de test/vérification en `--profile headless` (le Makefile le fait) ; `--release` seulement pour la capture.
- Disque : ~65 Go libres, **plancher 30**. `df -h /System/Volumes/Data` avant tout gros build ; sous 40 Go, `scripts/target-sweep.sh target` ; sous 30 Go, `ask`.
- Contrôle rapide : jamais `make check_rapide` d'un bloc (~47 Go de cible). Lance depuis ton worktree :
  `LOGS=$HOME/orca/runs/alacod/logs bash docs/taches/specs-coordinateur/scripts/p-1d-check-rapide-decoupe.sh`, puis `scripts/target-sweep.sh target`.
  Copie le journal sous `$HOME/orca/runs/alacod/logs/x-6f-check-rapide-<sha>.log`.
- **Suite complète jamais sur le Mac** : seulement tes scénarios et les familles du point (d). La suite tourne la nuit sur le VPS (`ssh ovh`) après le merge.
- Si `GITHUB_TOKEN` est dans ton environnement : `env -u GITHUB_TOKEN git push …` (le login valide est dans le trousseau).
- Ne touche jamais `/Users/wq/Project/bascanada/alacod` (checkout de William) ni `/Users/wq/Project/bascanada/1837` (checkout du coordinateur 1837). `1867_lore` : lecture seule.
- `pkill`/`killall` par motif interdits : arrête tes processus par PID. Aucun processus laissé à la fin.

## Règles du dépôt
`CLAUDE.md` (checklist GGRS : `Fixed`, `order_iter!`, `BTreeMap`, `despawn_rollback`, type rollback neuf en variante **neutre**, `impl Hash` manuel pour un champ ajouté
à un composant sous checksum, frontière simulation/présentation). Conventions : § **128** de `docs/conventions.md` pour ce lot ; dettes **D224–D225**.
Rapport `docs/taches/rapports/x-6f-changement-niveau.md` : par point de la preuve, issue (fait / pas fait / non vérifié) et chemin ; dernières lignes du contrôle
rapide ; ce qui reste. Ligne `LIVRÉ wquintal/x-6f-changement-niveau : …` à la fin de `docs/taches.md` (§ 10). Docs : `docs/conventions.md` § 128, et la liste des
attentes de `CLAUDE.md` si tu en ajoutes une (une ligne).

## Communication
- Branche `wquintal/x-6f-changement-niveau` (renomme la branche de ton worktree si elle porte un autre nom), poussée à chaque point de contrôle (avance rapide, jamais `--force`).
- Carte : `orca worktree set --worktree active --comment "<état en une ligne>" --json` à chaque étape franchie.
- Aperçu au coordinateur (`status`) à chaque étape franchie (relevé, scénarios rouges puis verts, capture) ; **si le lot dépasse deux jours, aperçu à mi-parcours obligatoire**
  (scénarios et une capture).
- Question bloquante : `ask` court avec ton choix par défaut, et avance sur le reste en attendant. Aucune question de diagnostic à William.
- Sur une erreur 429 / quota du fournisseur : `status` immédiat au coordinateur (il te bascule).
- Branche prête : `status` « X-6f prêt <sha> » avec le chemin du journal du contrôle rapide. Fin : rapport commité, branche poussée, **un seul** `worker_done` avec le rapport.
