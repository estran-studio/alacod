# Revue de X-6g (intérieur par graine) — consigne du relecteur, 21 nov 2026

Tu es **relecteur, lecture seule** : tu ne modifies aucun fichier, tu ne commites rien, tu ne lances **aucun cargo ni make** (disque du Mac à 35 Go, plancher 30). Français partout.

Branche : `origin/wquintal/x-6g-interieur-graine` @ `3975cb5e` (code et contrôle rapide sur `716cd159`, puis 3 lignes de rapport), contre `origin/main` (`18d576c4` ou plus récent).
Worktree : `~/orca/workspaces/alacod/x-6g-interieur-graine` (déjà sur la branche ; `git fetch` permis, rien d'autre qui écrit). Diff : `git diff origin/main...origin/wquintal/x-6g-interieur-graine`.

À lire : fiche `docs/taches/plan-1837-cli.md` § 6 ligne **X-6g** et § 3 (décisions du 21 nov, dont « générateur : extension minimale de la grammaire d'étage, aucun générateur neuf ») ;
rapport `docs/taches/rapports/x-6g-interieur-graine.md` ; `docs/conventions.md` § 129 (nouveau), § 40, § 17, § 128 ; `CLAUDE.md`. Journal du contrôle rapide :
`~/orca/runs/alacod/logs/x-6g-check-rapide-716cd159.log`.

## Ce que tu vérifies (dans cet ordre)
1. **Défaut inchangé au bit près** (P1 si violé) : sans `start_exits` / `exact` / `apart`, la grammaire du § 40 (`crates/map/src/generation/floor.rs`), `grid.rs`, `imp/basic.rs` consomment
   le RNG de génération exactement comme avant (même nombre et même ordre de tirages, mêmes nouvelles tentatives) ; étages `floor:` de gungeon/isaac et cartes `Basic` inchangés.
   Les 36 scénarios rejoués sans BLESS le soutiennent ; cherche ce qu'ils ne couvrent pas (ex. un champ par défaut qui change un ordre de tri, une condition de `validate_plan` évaluée pour tous).
   Les modifications de `imp/basic.rs` (+45) : que font-elles, le défaut est-il intact ?
2. **Déterminisme** : `BTreeMap`/listes ordonnées, aucun `HashMap`/`HashSet`, aucun `f32` dans la génération ; RNG consommé dans un ordre déterministe ; échec explicite borné (`Exhausted`), jamais
   de boucle infinie ; l'exception `std::time::Instant` ajoutée à `scripts/determinism-allowlist.json` (count 4 → 5) est bien hors simulation (test de campagne seulement).
3. **Conformité à la fiche et aux décisions** : salle centrale au centre avec un nombre **exact** de sorties ; types requis **exactement** ; deux types dans deux branches distinctes partant
   du départ (la définition de « branche » dans `validate_plan` est-elle juste : sous-arbres de deux enfants différents du départ, boucles comprises ?) ; graine par défaut rejouée à la relance
   (scénarios/test) ; graine explicite différente ; porte verrouillée refusée sans la clé ; enchaînement extérieur → porte (X-6f) → intérieur. Les 8 scénarios `interieur_*` testent-ils ce que
   leur nom dit (attentes, frames) ? Lint des nouveaux champs (combinaisons impossibles : `start_exits` > prises du gabarit de départ, `apart` sur un type non requis, etc.).
   Extension minimale : aucun générateur neuf, pas de code mort.
4. **Rien de 1837 dans l'engine** (noms neutres, gabarits du testbed) ; frontière simulation/présentation.
5. **Preuve** : campagne 200/200 (où, comment la rejouer), capture `docs/taches/rapports/preuves/x-6g/` (regarde les images et les PNG des six graines), tableau des traces.

## Rendu
`~/orca/runs/alacod/revue-x-6g-verdict.md` (hors dépôt, seule écriture permise) : verdict en une ligne (**merge** / **merge après corrections** / **ne pas merger**), puis P1 / P2 / P3 avec
`fichier:ligne`, problème, correction proposée. Puis `worker_done` avec ce fichier en rapport. Pas de question à William.
