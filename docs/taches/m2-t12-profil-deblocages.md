# m2-t12-profil-deblocages — profil et déblocages qui enrichissent le butin (M2, V1e, chantier G1)

Lire d'abord `docs/taches/README.md` (machine `orca`, §1 : `make sweep`, `make test_crates`,
signaling par session), `CLAUDE.md`, `docs/conventions.md` §38 (profil : `meta::Profile`, écriture
en fin de run, `ProfileHas`), §48 (tables de butin de M2-T5, flux `"loot"`), §50 (boutique, M2-T6,
si mergée), §33 (restart en ligne), et la décision `docs/plan-engine.md` §7 n° 8. Branche
`m2-t12-profil-deblocages` depuis `main` (M2-T5 mergée). Estimation : 3 j. Section de conventions
réservée : **§51**.

## Contexte

Le profil est écrit en fin de run (essence, stats, `first_run`/`first_victory`) mais **la
simulation ne le lit jamais**. Ici, les déblocages entrent dans la partie : ils ajoutent des objets
aux pools de butin. Le piège : en p2p, chaque pair a **son** profil ; si chacun enrichit les pools
avec ses propres déblocages, les tirages divergent (desync immédiat au premier coffre).

## À faire

1. **Règles de déblocage en données** : kind de contenu `Unlock` (`unlocks/<id>.ron`) : coût en
   monnaie méta ou condition (stat ≥ n, déblocage préalable), et ce qu'il ajoute (objets à une
   table de butin, `loot/<table>` : entrées supplémentaires). Remplacer les règles provisoires de
   §38 par ces données (essence gagnée : données d'`economy.ron` du jeu).
2. **Déblocages de la partie** : un ensemble `RunUnlocks` (`BTreeSet<String>`) **figé au lancement
   de la run** et identique sur tous les pairs : en local, union des profils des joueurs locaux ;
   en ligne, celui de l'hôte (le pair de plus petit handle) transmis avec la configuration de
   session comme la graine (ou intersection des profils si c'est plus simple à transmettre : le
   justifier). Ressource rollback (statique pendant la run) ; les tables de M2-T5 l'appliquent au
   tirage, sans toucher au flux `"loot"` quand l'ensemble est vide (traces inchangées).
3. **Achat de déblocages** : hors simulation (menu du lobby ou commande `alacod-profile unlock
   <id>` ; pas de PNJ, c'est M2-T13), débite la monnaie méta, écrit le profil (écriture atomique
   existante).
4. **Lint** : déblocage vers un objet ou une table inconnus, coût ≤ 0, cycle de prérequis →
   refusés ; fixtures.
5. **Contenu `gungeon`** : trois déblocages (deux objets de `M2-T4`/`M2-T5` retirés des pools de
   base et rendus déblocables, un coffre plus riche) ; `zombies`/`testbed`/`throne` inchangés.
6. **Scénarios** : `unlock_enriches_pool` (même graine, avec/sans déblocage : objet différent au
   coffre), `unlock_purchase` (achat hors simulation, `ProfileHas`), `unlock_two_players_host`
   (deux joueurs aux profils différents : `RunUnlocks` = règle choisie), et un test p2p à deux
   clients aux profils différents sans desync (`make test_multiplayer` ou le runner p2p headless).

## Règles

Aucune trace existante ne doit bouger (ensemble vide = comportement d'avant ; nouveaux types en
variante **neutre**). Le profil n'est jamais lu dans `GgrsSchedule` en dehors de `RunUnlocks`.
Pas de `f32`, `order_iter!`, BTreeMap. `cargo fmt --all -- --check` avant de livrer.

## Hors périmètre

Hub et PNJ marchand (M2-T13), synchronisation en ligne des profils (pubkey allumette), méta-monnaie
dépensée en run.

## Vérification

README §4 (`make test_scenarios`, `make test_crates`, lint ×4, fmt, scripts, `make gen` ×4) ;
p2p à deux clients aux profils différents sur `gungeon` (port 3536, `MATCHBOX_PORT`) ; rapport
`docs/taches/rapports/m2-t12-profil-deblocages.md`, conventions §51. Livrer par
`LIVRÉ m2-t12-profil-deblocages <sha>`.
