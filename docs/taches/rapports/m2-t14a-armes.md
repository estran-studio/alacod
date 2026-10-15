# Rapport m2-t14a-armes (M2-T14, partie a : armes `gungeon`)

## 1. Fait
- `games/gungeon/assets/weapons/weapons.ron` : 18 armes à distance ajoutées devant `arsenal` (le lint
  compte maintenant **20 armes** : `pistolet`, 18 nouvelles, `arsenal`; plus `bare_hands`). La fiche
  parle de « quatorze de plus » : le compte du lint partait de 2, donc quatorze n'atteignait que 16 ;
  j'en ai écrit 18 pour que le lint affiche vingt. Contenu : revolver, pistolet_precis, pistolet_jumele,
  carabine_rafale, mitraillette, mitrailleuse_lourde, rebondisseur, fusil_pompe, double_canon,
  fusil_precision, arc, arbalete, laser, chercheur (Homing), boule_energie, lance_roquettes, grenadier,
  lance_flammes. Munitions `Custom` par type (balles, cartouches, calibre, fleches, energie, roquettes,
  carburant). Uniquement des comportements existants ; aucune mécanique manquante à signaler.
- Chaque arme porte `test:` avec `min_hits`/`max_hits` mesurés (≈ 0,6× et 2×+4 du nombre de coups observé).
- 18 scénarios générés `tests/scenarios/generated/gungeon/weapon_*.ron/.trace` (`make gen GAME=gungeon`).
- `docs/conventions.md` §52.a.

## 2. Vérifié (résultats réels)
- `make lint` : 4 jeux « aucune erreur » (gungeon : 20 armes).
- `make gen GAME=gungeon` : 23 scénarios `ok`, tous traces présentes après bless ; second passage sans bless :
  tout `ok`/`ok`, `git status` propre. Les 5 traces gungeon préexistantes n'ont **pas** bougé après
  `GEN_BLESS=1` (diff git vide). `make gen` zombies/testbed/throne : aucun diff.
- `make test_scenarios` (série, profil headless) : `test result: ok. 3 passed; 0 failed; 8 ignored`, 3162 s.
- `check-rollback-registration.sh` : OK ; `check-forbidden.sh` : avertissements préexistants seulement.
- Calibrage : un premier `lance_flammes` (portée 140) faisait 0 coup ; portée 190, vitesse 300, vie 40 → 40 coups.

## 3. Non fait / non vérifié
- **p2p gungeon à deux clients** : pas de Docker ici, à rejouer par l'orchestrateur. Aucun code de
  simulation modifié, mais non vérifié.
- `make test_crates` : non lancé (aucun Rust modifié). `cargo fmt --check` : rustfmt absent du toolchain du
  conteneur, non lancé (aucun `.rs` modifié).
- `make test_scenarios_par` échoue ici (le script cherche ses journaux sous un chemin relatif après `cd`) ;
  j'ai utilisé `make test_scenarios`. Bench (`ALACOD_BENCH_STRICT`) non fait.

## 4. Dettes
- Pas de munition infinie dans le moteur : seul `pistolet` a une réserve de 9999 chargeurs.
- Sprites : toutes les armes réutilisent le sprite `pistol` (placeholder).
- Les armes ne sont placées nulle part (butin, boutique, ramassage) : à brancher par T14b/T14d.

## 5. Atteignabilité
Les armes ne sont atteignables que par les scénarios générés `weapon_*` (arme imposée au joueur).
Dans une partie ordinaire du clone le joueur n'a que `pistolet` : le butin d'armes (tables de M2-T5/T6) sera
branché par T14b (objets) et T14d (gabarits, coffres, boutique) ; dette nommée.
