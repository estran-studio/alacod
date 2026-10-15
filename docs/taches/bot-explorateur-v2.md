# bot-explorateur-v2 — l'explorateur joue l'arène et réanime (D65)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/conventions.md` §47 (bot explorateur), §45
(arène), le rapport `docs/taches/rapports/m2-t16-campagne-boss.md` et ses scripts
(`docs/taches/rapports/m2-t16-campagne-boss/campagne.sh`, `analyse.py`, `morts.py`), `crates/bots`.
Branche `bot-explorateur-v2` depuis `origin/main`. Estimation : 2 j. Compléter §47.

## Contexte

La campagne contre le boss (D65) a montré deux trous du profil `explorateur` :
- il ne vise que les salles `combat` et `boss` : l'**arène** exigée par la grammaire `etage` n'est
  jamais jouée, ni la boutique, ni la récompense. Aucune campagne ne mesure donc ces salles ;
- à 2 bots, deux **soft-locks de réanimation** (`etages_deux`, graines 3 et 175) : un bot vivant, à
  côté de son coéquipier à terre, ne le réanime pas et ne progresse plus.

## À faire

1. Buts de l'explorateur : toutes les salles verrouillables (arène comprise), et passage dans les
   salles `recompense` et `boutique` (sans acheter : entrer suffit). Ordre de visite déterministe.
2. Réanimation prioritaire : un coéquipier à terre est le premier but tant qu'il peut être réanimé.
3. Inspecter la graine 175 et dire au rapport ce qui la bloquait.
4. Campagnes `campagne.sh` b2 et e2 (2 bots) sur 200 graines : 0 soft-lock, 0 desync ; arène
   verrouillée puis nettoyée sur au moins une graine nommée. b1 et e1 rejouées pour comparer (le taux
   de mort de `etages_deux` relève de D64, pas de cette tâche : le chiffrer seulement).

## Règles

Le bot est de l'outillage : la simulation du jeu ne change pas. Les traces des scénarios joués par
l'explorateur (`bot_explorer_*`, `room_*` générés…) bougent : preuve §5, cause dans le commit (« le bot
choisit d'autres buts »). Pas de `f32`, `order_iter!`. Ne pas toucher au contenu ni au boss.
`cargo fmt --all -- --check`.

## Session cloud

Tâche prévue pour une session Claude Code cloud : ignorer les consignes `orca` du README (§1) ; pas
de Docker (p2p non requis : les bots jouent hors réseau). Campagnes au premier plan, par lots de 10
graines poussés un par un (le conteneur redémarre environ toutes les deux heures). Voie rapide du
README §6 : fusionner `origin/main` et rejouer la suite juste avant le commit
`LIVRÉ bot-explorateur-v2`, avec le rapport `docs/taches/rapports/bot-explorateur-v2.md`.

## Vérification

README §4 (`make test_scenarios` complet, `make test_crates`, `make lint` ×4, fmt, scripts,
`make gen` ×4) ; campagnes au rapport.
