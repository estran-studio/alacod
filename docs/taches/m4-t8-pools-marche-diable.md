# m4-t8-pools-marche-diable — pools par type de salle, épuisement, marché du diable (M4, V1b, C4 v2 / C5 v2)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/taches.md` §8.1, `docs/conventions.md` §48, §50, §51
(déblocages de M2-T12), §61 (`Price`), §64 (pools à épuisement), §70 (salle post-boss). Branche
`m4-t8-pools-marche-diable` depuis `origin/main`, **après le merge de M2-T12, M4-T0e, M4-T3 et M4-T5**, et
les réponses de William à **Q5** (épuisement) et **Q6** (diable). Estimation : 4 j. Section de conventions
réservée : **§73**.

## À faire

1. **Pool par type de salle** (`tresor`, `boutique`, `boss`, `diable`, `secrete`), tirés selon la règle
   d'épuisement retenue en Q5 (proposition de T0e : tirage à la génération de l'étage, objet tiré retiré pour
   la run) ; `RunUnlocks` de M2-T12 appliqué comme pour les autres pools.
2. **Présentoir payé en vie max** : `Price::MaxHealth` exécuté (achat par Interaction, refus si le paiement
   tue ou laisse moins que le seuil retenu en Q6) ; `Price::Consumable` aussi.
3. **Salle du diable** : contenu de la salle post-boss de T5 (deux ou trois présentoirs du pool `diable`,
   prix en vie max), chance d'apparition selon Q6 (proposition : donnée par étage, réduite si le joueur a
   subi des dégâts au boss : drapeau dérivé du `RunSummary` de l'étage, à confirmer).
4. **Scénarios** : `pool_exhaust_two_rooms`, `pool_exhaust_time_independent` (deux ordres de visite, mêmes
   objets si la règle le promet), `devil_deal_pay_health`, `devil_deal_refused_lethal`,
   `devil_room_chance` (graine où elle apparaît), `pool_unlock` (déblocage de M2-T12 visible dans le pool).

## Règles

Pools sans `exhaust` et boutiques à prix en sous : comportement et traces d'avant. RNG local (§48).

## Hors périmètre

Salles d'ange (Q6), contenu des pools (T11d), HUD des prix en cœurs (T16).

## Vérification

README §4 ; rapport `docs/taches/rapports/m4-t8-pools-marche-diable.md`, conventions §73.

## Atteignabilité

Salle de trésor (une par étage) et boutique exigées par la grammaire de `isaac` ; salle du diable : graine
nommée où elle apparaît après le boss de l'étage 1 et où le bot (T14) prend un objet.

## Session cloud

Tâche prévue pour une session Claude Code cloud, **après le merge de M2-T12**. Ignorer les consignes `orca`
du README (§1). Pas de Docker : le dire au rapport. Pousser la branche régulièrement. Voie rapide du README
§6 : fusionner `origin/main` et rejouer la suite juste avant le commit `LIVRÉ m4-t8-pools-marche-diable`, qui
contient le rapport.
