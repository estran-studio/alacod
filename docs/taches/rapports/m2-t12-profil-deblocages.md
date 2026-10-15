# M2-T12 — Profil et déblocages

Branche `m2-t12-profil-deblocages`. Conventions : `docs/conventions.md` §51.

## Ce qui est livré

- **Kind `Unlock`** (`unlocks/<id>.ron`) : prix en essence, prérequis, statistique minimale, et ce
  qu'il accorde (`Item` : l'objet quitte les pools tant que le déblocage manque ; `Loot` : une entrée
  de plus dans un pool). Kind `MetaEconomy` (`meta/essence.ron`) : essence par run, par kill, par
  victoire (les règles provisoires de §38, mêmes valeurs).
- **`RunUnlocks`** : ensemble figé au lancement, ressource rollback **neutre** (vide = contribution 0,
  traces inchangées). Local : union des profils des joueurs locaux, **limitée aux `Unlock` du jeu**.
  En ligne : le profil de l'hôte, transmis avec la configuration de session (`ALACOD_RUN_UNLOCKS`).
- **Tirage** (`draw_loot_with`) : retire les objets cachés, ajoute les entrées des déblocages actifs ;
  le RNG consomme toujours ses deux `next_u32` : même graine, même suite de tirages, seul l'objet change.
- **Achat** hors simulation : `Profile::purchase` (déjà possédé, prérequis, statistique, essence) et
  `alacod-profile unlock games/<jeu> <id>` (`list` : état).
- **Lint** : objet / table / prérequis inconnus, prix ≤ 0, cycle ; cinq fixtures.
- **Gungeon** : `tresor_trefle` (20), `etendard_royal` (40, `runs` ≥ 1), `coffre_riche` (60, exige le
  trèfle, entrée Rare dans `commun`). Zombies, testbed, throne : inchangés.
- **Scénarios** (sur `etage_1`) : `unlock_enriches_pool` / `unlock_base_pool` (même graine : avec les
  trois déblocages, « amulette » ×2 ; sans, « amulette » + « bottes »), `unlock_purchase`,
  `unlock_two_players_host` (union locale).

## Décision : RunUnlocks limité aux `Unlock` du jeu

Le profil gagne des jalons (`first_run`, `first_victory`) à chaque fin de run. Relire tout le profil à
la relance (`RunRequest::Restart`) mettait ces identifiants dans `RunUnlocks` : l'état différait dès la
frame 0 et `restart_replays_identically_under_ten_seconds` échouait. Le filtre sur `registry.unlocks`
règle ça sans désactiver le test. Conséquence : une relance voit un déblocage **acheté entre deux runs**
(achat hors simulation) ; ce n'est pas un jalon, et c'est voulu. Le test n'achète rien.

## Preuve §5

Aucune trace existante n'a bougé (suite complète verte avant et après les fusions de main). Les quatre
traces `unlock_*` sont nouvelles.

## Pièges rencontrés

- Une seule Interaction ouvre le coffre mais ne ramasse pas l'objet : il en faut une seconde.
- Gungeon démarre sur `etages_deux` depuis D50/D54/D59 : les scénarios T12 fixent `floors: Some("etage_1")`.
- Conflits de fusion : conventions (§50 boutique / §51) et fixtures de lint (boutique / déblocages).

## Non fait (hors périmètre)

Hub et PNJ marchand (M2-T13), synchronisation en ligne des profils (pubkey allumette), méta-monnaie
dépensée en run. En ligne, l'ensemble vient de la variable d'environnement, pas encore d'un échange réseau.
