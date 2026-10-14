# m2-t16-bot-explorateur — bot explorateur d'étages (M2, V3, chantier K2 v1)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/conventions.md` §35 (salles, verrous),
§36 (objets au sol), §40 (étages assemblés, portail après le boss), §41 (blank), les profils de
`crates/bots` (`hunter.rs`, `navigation.rs`, `dodge.rs`, `stuck.rs`) et la cible `make sim`
(`alacod-sim`, `crates/scenario/src/runner.rs` : `until_wave`). Dette **D59** (`docs/taches/dettes.md` :
la grammaire d'étage n'est jamais jouée en conditions réelles). Branche `m2-t16-bot-explorateur`
depuis `main`. Estimation : 4 j. Section de conventions réservée : **§47**.

## Contexte

Les bots savent chasser, réparer, acheter, esquiver (profils `fonceur`, `prudent`, `acheteur`…)
sur des cartes à vagues. Dans `gungeon`, l'atteinte du boss est encore **scriptée** dans les
scénarios d'étage (`gungeon_floor_seed_*`). Il faut un bot qui joue un étage assemblé seul :
c'est l'outil qui rend testables la grammaire (D59), le butin (M2-T5) et le boss (M2-T7).

## À faire

1. **Profil `explorateur`** (`BotProfile`, variante en fin d'enum) : choisit la salle non visitée
   la plus proche par le chemin (flow field / graphe de salles, ordre déterministe : distance puis
   id de salle), y entre, nettoie (réutilise la chasse et l'esquive existantes, blank quand une
   balle ennemie est à moins de `n` px et qu'il en a), ramasse les objets au sol, va au boss quand
   tout est visité (ou directement si une option le demande), prend le portail.
2. **Mémoire du bot** : salles visitées dérivées de l'état (position + `RoomState`), pas d'état
   caché hors rollback (piège « donnée dérivée non rollbackée » de `CLAUDE.md`).
3. **`alacod-sim --until-floor N`** (comme `--until-wave`) + sortie JSON : étage atteint, salles
   visitées, morts, frames, desync, soft-lock (même détecteur que `crates/scenario/tests/softlock.rs`).
4. **Campagne** : `alacod-sim --game gungeon --bots 1 --profiles explorateur --seeds 1..200
   --until-floor 2` : objectif 200 graines sans soft-lock ni desync ; les échecs (mort, blocage)
   classés dans le rapport avec la graine et la cause. Faire de même à 2 bots.
5. **Scénarios** : `bot_explorer_floor` (un étage `court` joué par le bot, `RoomState` `Cleared`
   sur toutes les salles, `FloorIndex` 1), `bot_explorer_two` (deux bots). Ils remplacent à terme
   l'atteinte scriptée du boss ; **ne pas** modifier les scénarios `gungeon_floor_seed_*` existants.
6. Mettre à jour D59 dans `dettes.md` avec ce que la campagne montre.

## Règles

Les bots produisent des inputs : aucune trace existante ne doit bouger (les profils existants
inchangés). Pas de `f32` dans la décision (`Fixed`), ordre `GgrsNetId`, BTreeMap, sélection avec
tie-break déterministe. `cargo fmt --all -- --check` avant de livrer.

## Hors périmètre

Bot de boutique (M2-T6), parade, intelligence de build (choix d'objets), équilibrage.

## Session cloud

Tâche prévue pour une session Claude Code cloud (pas la machine `orca`) : ignorer les consignes
`orca` du README (§1 : `make sweep`, ports de signaling) ; build sans `game/dev-dylib` si
`./scripts/dev-features.sh` ne le propose pas. La session ne peut pas écrire à l'orchestrateur :
**pousser la branche régulièrement** (commits WIP acceptés). Le conteneur cloud **redémarre
environ toutes les deux heures** et tue les processus d'arrière-plan (constaté par la session
`m0-200-graines-test-map`, 2026-10-05) : la campagne se joue **au premier plan, par lots de 10
graines au plus**, chaque lot écrit son JSON et est committé/poussé avant le suivant ; si elle ne
tient pas, s'arrêter à ce qui est mesuré et le dire au rapport (l'orchestrateur finira sur `orca`) et finir par un commit
`LIVRÉ m2-t16-bot-explorateur` qui contient le rapport. p2p : `make test_multiplayer` si docker et
un affichage sont disponibles, sinon le dire au rapport.

## Vérification

README §4 (`make test_scenarios` complet sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
`./scripts/check-forbidden.sh`, `./scripts/check-rollback-registration.sh`, `make gen` ×4) ;
campagne 200 graines (JSON résumé au rapport) ; rapport `docs/taches/rapports/m2-t16-bot-explorateur.md`,
conventions §47.
