# X-6d « S'agripper au joueur » (`Grab`) — sur le VPS (coordinateur alacod, run `run_9effecfcae39`, ven 20 nov 2026)

Objectif studio O2 au service d'O3. Jalon **S4 dim 6 déc** (« lutins qui s'agrippent »), parti en avance. Simulation : checklist GGRS de `CLAUDE.md` (fait foi). Tu ne merges pas.
Preuve qui fait foi : ligne **X-6d** du § 6 de `docs/taches/plan-1837-cli.md` (`d63deb33`). § 126, D222, rapport `docs/taches/rapports/x-6d-agrippeur.md`.

## Le besoin (design, lecture seule)
`/home/debian/workspace/1867_lore/design/archetypes-ennemis.md` (« Petite verrue agrippante ») et `design/production/lutin.md` (brouillon : durée maximale accroché, force de la
gêne, nombre à la fois **ouverts** → données réglables, valeurs du testbed neutres et raisonnables, jamais codées dans l'engine). Boucle : court vers le joueur, **bondit**,
**s'accroche** sur ses épaules, **gêne** son déplacement (ralenti ou dévié), presque pas de dégâts ; réponse : **secouer** (stick ou équivalent clavier) ou **coup de mêlée** ;
décroché, il **retombe étourdi** (ouverture de frappe). Composition : dangereux quand d'autres ennemis forcent à bouger ; le joueur ne perd jamais longtemps le contrôle.
**Rien de 1837 dans l'engine** : nom neutre (ex. `Grab`/agrippeur), personnage de testbed neutre ; 1837 posera ses lutins en données plus tard (décris au rapport ce qu'il
lui faudra : champs du personnage, poses).

## Ordre
1. Lis ce qui existe avant d'écrire : `crates/behaviors` (règles, `BehaviorState`), attaches/ancres existantes (familiers M3-T6 § 71, porteur de clé X-6a1 § 117, `Charge`
   zigzag X-6b § 119, `TakeCover` X-6c § 120), modificateurs de vitesse du joueur (statuts § 19), entrée du joueur (pour le secouage : lire l'input rollback, jamais le clavier
   en dehors), `ModelPlayback`/poses (§ 122) et attentes T-3b (§ 123). Réutilise ; ne crée pas un second mécanisme d'attache s'il en existe un.
2. Scénarios et attentes d'abord (rouges), puis le behavior. Liste minimale (preuve du § 6) : bond + accroche ; gêne mesurée sur la vitesse du porteur ; décrochage au stick
   **par l'entrée réelle** (harnais T-3b `gestes_entree_reelle`) et au clavier ; décrochage à la mêlée ; étourdi puis frappe ; durée maximale ; deux agrippeurs ; coop (le coéquipier
   frappe l'agrippeur). Chaque attente nouvelle **échoue sur une régression provoquée** (journal). Poses `saut`, `accroche` (sur l'ancre des épaules du porteur, présentation),
   `etourdi` : lint `MissingPose` et `animation_vivante` les couvrent.
3. Un scénario court étiqueté `rapide` si ça tient dans le budget de `check_integration`. Vidéo (`make videos SCENARIO=…`, sous `xvfb-run -a`) des scénarios principaux.
4. `make check_sim_boundary`, `make schedule_order_diff BASE=origin/main`, contrôle rapide découpé (sur mon feu vert), rapport, ligne LIVRÉ.

## Où, règles
- Worktree `/home/debian/orca/workspaces/alacod/x-6d-agrippeur`, branche `wquintal/x-6d-agrippeur`. Cible : `CARGO_TARGET_DIR=/home/debian/orca/workspaces/alacod/target-partage`
  (chaude ; le coordinateur 1837 peut l'emprunter : verrou de cargo partagé, normal ; il ne balaie pas). Un autre worker engine (P-1e) a sa cible `target-p1e` : n'y touche pas.
- Préfixe `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1`. Un seul cargo ; jamais de profil debug ; cargo seulement si `df -h /` ≥ 20 Go libres, arrêt de
  TON cargo sous 15 Go et `status` ; `scripts/target-sweep.sh $CARGO_TARGET_DIR` après chaque cargo (jamais pendant un cargo d'un autre : `pgrep -af target-partage` d'abord) ;
  rien de lourd entre 03:45 et 05:00 UTC (suite de nuit).
- Deux contrôles rapides ne tiennent pas ensemble sur le disque : **avant ton contrôle rapide découpé, `ask` « contrôle rapide ? »** et attends mon feu vert. Scripts :
  `~/orca/runs/alpha/logs/p-1d-check-rapide-decoupe.sh` et `p-1d-test-crates-decoupe.sh` sous ton nom (journal `logs/x-6d-agrippeur-check-rapide-<sha>.log`), puis purge des
  binaires de tests d'intégration et sweep.
- Aucune trace existante modifiée ; ne `BLESS` que les traces neuves de ton lot. Lis `CLAUDE.md`, `~/orca/runs/alpha/commun.md` (écrit pour le Mac ; ce fichier-ci prime).

## Communication
Commite et pousse à chaque étape. Carte `orca worktree set --worktree active --comment …`. `status` à chaque étape (rouges, vert, E prêt) et au moins une fois par jour ;
aperçu à mi-parcours (voie de plus de deux jours) : une vidéo ou un journal de scénario. 429 : `status` tout de suite. Question bloquante : `ask` avec ton choix par défaut,
et avance. Fin : rapport, LIVRÉ, aucun processus, un seul `worker_done`.
