# m1-d26-doublons-generes — scénarios générés en double entre testbed et zombies (D26) ; D27 close

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b1, branche
`m1-d26-doublons-generes` créée depuis `origin/main`, même target). Outillage V3, ½ j. Compilation
après le **feu vert** d'orch, `CARGO_BUILD_JOBS=2`.

## Contexte

D26 : `tests/scenarios/generated/testbed/` contient des scénarios d'armes qui doublonnent ceux de
`generated/zombies/` (mêmes armes, même arène), ~80 s de plus par `make test_scenarios`. Cause : le
générateur (`crates/scenario/src/generate.rs`, binaire `alacod-gen`) produit un gabarit par arme du
**registre fusionné** du jeu, et le testbed hérite des armes de zombies. D27 (dispersion ignorée) est
le même défaut que D51, fermé le 2026-10-06 : à fermer comme doublon.

## Décisions (fixées ici)

1. **Règle** : `make gen GAME=g` ne génère que les définitions **déclarées par le jeu g lui-même**
   (fichiers sous `games/g/assets/`), pas celles héritées d'un autre jeu. Si le registre ne garde pas
   l'origine d'une définition, l'ajouter hors simulation (chemin du fichier source déjà connu du
   chargeur : `RegistryEntry.file` ou équivalent) ; aucun changement de hash ni de trace.
2. **Avant de supprimer** : lister exactement les scénarios `generated/testbed/*` qui disparaissent et,
   pour chacun, le scénario `generated/zombies/*` qui couvre la même définition (même arme, même
   gabarit) ; un scénario testbed sans équivalent ne disparaît pas (si c'est le cas, la règle 1 est
   fausse pour lui : le dire).
3. Les `.ron`/`.trace` supprimés le sont par `make gen` (pas à la main), le test `generated_up_to_date`
   (ou équivalent) reste vert, et `make gen` ×3 sans modification après coup.
4. Mesurer le temps de `make test_scenarios` avant/après sur une machine calme si possible (sinon dire
   sous quelle charge).
5. `dettes.md` : ne pas toucher (orch ferme D26 et D27).

## Livrer

Suite complète (aucune trace existante modifiée, seulement des suppressions listées), crates, lint ×3,
fmt, scripts, exemples ; rapport `docs/taches/rapports/m1-d26-doublons-generes.md`. Merger
`origin/main` juste avant. `git push -u origin m1-d26-doublons-generes`, puis `SendMessage` à `orch` :
`LIVRÉ m1-d26-doublons-generes <sha> : <n scénarios supprimés, temps avant → après>`.
