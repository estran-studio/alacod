# m3-t0b-contrats-objets-sante — exemplaires multiples, prix, santé et pickups (M3, vague 0)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/taches.md` §8.1, `docs/conventions.md` §4,
§12 (monnaie), §18 (D61 : armure et bouclier), §36 et §44 (objets, synergies, consommables), §42 (effets
v2, clés de limites `item:<id>:<index>`), §48 et §50 (butin, boutique, `shells`), et le code
`crates/items/src/lib.rs` (`Inventory`, `ItemDef`), `crates/game/src/items.rs` (ramassage),
`crates/game/src/effects_runtime.rs` (`listed_effects`, clés de limites), `crates/effects/src/` (`Effect`,
`Action`, `runtime::Limits`), `crates/combat/src/actors.rs` (`Health`, `Shield`),
`crates/game/src/character/health/mod.rs` (`rollback_resolve_damage_events`), `crates/run/src/currency.rs`.
Branche `m3-t0b-contrats-objets-sante` depuis `origin/main`. Estimation : 2 à 3 j. Section de conventions
réservée : **§61**.

## But

Poser les contrats dont dépendent M3-T2 (effets v3), M3-T3 (cœurs et pickups) et M3-T8 (marché du diable),
avec un chemin minimal dans le testbed. **Aucune trace existante ne bouge.**

## Contexte (état de `main`)

- Un passif ramassé deux fois est poussé deux fois dans `Inventory.passives` et ses modificateurs deux fois
  dans `Modifiers` (même source `item:<id>`) : les stats s'additionnent déjà par exemplaire.
- Ses **effets**, eux, sont listés deux fois avec la **même clé** de limites `item:<id>:<index>` : sans
  limite, chaque exemplaire se déclenche ; avec `cooldown`/`charges`, les deux partagent la limite.
  Conséquence : `amulette` et `gilet` (gungeon, `OnPickup` + `charges: Some(1)`) ne donnent rien au second
  exemplaire. Rien ne dit combien d'exemplaires un joueur tient.
- Prix : `Shelf.price` est un `u32` de `Currency` (shells) ; aucun prix en vie ou en consommable.
- Santé : `Health { current, max, invulnerable_until_frame }` en `Fixed`, `Shield(u32)` (un point absorbe
  un coup entier, i-frames de 15 frames), `Armor`. **Aucune i-frame après un coup** pour un joueur : seuls le
  dash, le blank et le bouclier en posent.
- Consommables : compteurs `BTreeMap<String, u32>` sans plafond (`key`, `blank`) ; pas de `bomb`.

## À faire

1. **Compte d'exemplaires** : `Inventory::count(id) -> u32` (passifs ; l'actif compte pour un). Pure, testée.
2. **Limites par exemplaire** : la clé de limites du k-ième exemplaire d'un objet devient
   `item:<id>#<k>:<index>` pour `k ≥ 1` ; le premier exemplaire garde `item:<id>:<index>` (hash d'`EffectState`
   inchangé pour tout joueur qui n'a qu'un exemplaire). **Contrat seulement** : la fonction de clé et ses
   unitaires ; le basculement de `listed_effects` est M3-T2 (il change le comportement d'un second
   exemplaire, donc des parties gungeon où un doublon est tiré : preuve §5 là-bas). Le principe « chaque
   exemplaire a ses limites » est la proposition par défaut ; **Q16** (au rapport) demande à William s'il le
   veut pour tous les objets ou par objet (`stack: Independent | Shared`).
3. **Ampleur par exemplaire** : champ optionnel d'action `scale: Option<Scale>` avec `Scale::PerCopy` (la
   valeur est multipliée par `count(id)` de l'objet porteur de l'effet), en fin de structure et hors hash
   quand absent. Contrat : forme RON, lint (seulement sur un effet d'objet), règle pure ; exécution en T2.
4. **Prix** : `items::Price { Currency(u32), Consumable(String, u32), MaxHealth(Fixed) }` (enum à trois
   variantes dès sa création : pas de piège de discriminant). `Shelf` garde son champ `price: u32` (hash
   inchangé) et reçoit `cost: Option<Price>` hors hash quand absent ; `Price::Currency` équivaut à `price`.
   Règles pures `can_pay`/`pay` (vie max : refus si le paiement tuerait, Q6). Exécution en T8.
5. **I-frames après un coup** : champ optionnel `hit_iframes: Option<u32>` de `CharacterConfig` ; quand il est
   posé, le résolveur de dégâts écrit `Health.invulnerable_until_frame = frame + n` après un coup **subi**
   (montant > 0, après bouclier et armure). Absent partout dans le contenu existant : aucune trace ne bouge.
   C'est un vrai changement de résolveur : unitaire du résolveur et scénario `hit_iframes` (testbed : deux
   balles à 10 frames d'écart, la seconde ne touche pas avec `hit_iframes: Some(30)`).
6. **Couche de vie au-dessus des PV** : **bloqué par Q2** (modèle des cœurs). Proposition par défaut à
   présenter, non à coder avant la réponse : une demi-case de cœur = 1 point de `Health` (le contenu donne
   des PV pairs, des dégâts de 1 ou 2), et un composant neutre `ExtraHealth { soul: Fixed }` absorbé
   **avant** les PV, point par point (≠ `Shield` qui absorbe un coup entier). Si William répond avant la fin
   de la tâche : contrat de `ExtraHealth` (composant neutre, ordre dans le résolveur, attente
   `ExtraHealth(handle, n)`) ; sinon, le dire au rapport et laisser ce point à T3.
7. **Consommable `bomb`** : rien de spécial côté contrat d'objet (un consommable compteur comme `key`) ;
   réserver le nom `game::loot::BOMB_ITEM = "bomb"` (constante, comme `KEY_ITEM`), utilisé par M3-T0c/T4.
8. **Plafond de compteur** : champ optionnel `max_stack: Option<u32>` d'un consommable (Isaac : 99), absent =
   sans plafond (comportement actuel). Ramassage au-delà du plafond : l'objet reste au sol. Contrat + lint ;
   exécution en T3.
9. **Partage en coop** : **bloqué par Q4** (sous, bombes, clés partagés par l'équipe ou par joueur). Ne rien
   coder ; décrire au §61 les deux options et ce que chacune change (`Inventory` par joueur vs ressource
   d'équipe, ordre de ramassage, prompts).
10. **Lint et kinds** : `Price` (consommable connu, montant > 0, vie max dans ]0, max]), `scale` hors objet,
    `max_stack = 0`, `hit_iframes = 0` ; fixtures `crates/content/tests/fixtures/` ; attentes nouvelles dans
    `replay.rs` et la liste de `CLAUDE.md` (bloc « Attentes », ligne M3) : `ItemCount(handle, id, n)` existe
    déjà (M2-T5) ; ajouter seulement ce qui manque (`Invulnerable(handle, at_frame)` si utile au scénario 5).

## Règles

Aucune trace existante ne bouge : nouveaux types en variante neutre, champs optionnels hors hash quand
absents, `impl Hash` manuel s'il le faut (comme `Effect` en §42). Le changement de clé de limites n'est
**pas** branché ici (point 2). Pas de `f32`, `order_iter!`, `BTreeMap`, `FrameEvents`.

## Hors périmètre

Exécution de `PerCopy`, des limites par exemplaire et des transformations (M3-T2) ; sous et bombes au sol,
couche de vie en jeu, plafonds en jeu, partage en coop (M3-T3) ; marché du diable (M3-T8) ; HUD (M3-T16).

## Prérequis

Lançable tout de suite pour les points 1 à 5, 7, 8 et 10. M2-T12 touche aussi `crates/items` (tirage des
pools avec `RunUnlocks`) : fusionner `origin/main` juste avant de livrer, conflits additifs attendus.

## Vérification

README §4 complet ; scénario `hit_iframes` et unitaires (compte, clé, `Price`, `Scale`) ; rapport
`docs/taches/rapports/m3-t0b-contrats-objets-sante.md`, conventions §61.

## Atteignabilité

Contrats : rien n'est encore rencontré en partie ordinaire. `hit_iframes` sera posé par le personnage de
`isaac` (T0f/T3) ; `Price::MaxHealth` par la salle du diable (T8).

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de
Docker : p2p rejoué par l'orchestrateur, le dire au rapport. Pousser la branche régulièrement. Voie rapide du
README §6 : fusionner `origin/main` et rejouer la suite juste avant le commit
`LIVRÉ m3-t0b-contrats-objets-sante`, qui contient le rapport.
