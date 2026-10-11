# Mutations entre étages — 2026-10-10

William a accepté le plan des correctifs, dont cette politique, le 2026-10-10.
Throne en mode Floors met les choix gagnés en attente pendant le combat. Après
nettoyage et ouverture du portail, tous les choix sont présentés ; le portail
attend les joueurs. Les balles/hitboxes hostiles restantes sont supprimées et
les dégâts résiduels sont ignorés pendant l'interlude. Le timeout historique de
600 frames par choix reste disponible. Les horloges continuent : la simulation
ne se met pas en pause. Les autres jeux et le mode Sandbox gardent Immediate.

Validation ciblée : deux joueurs gagnent trois niveaux à f10, aucun choix ne
s'ouvre pendant le combat, aucune mutation n'est perdue ; les deux terminent
leurs trois choix avant la transition. Le test injecte des dégâts létaux à
chaque frame où une carte est ouverte : l'interlude reste sûr. Synctest distance 2.

Preuve détaillée : `throne_deferred_choice_trace_starts_at_the_earned_level`
rejoue le premier étage avec les deux politiques, sans modifier les fichiers
RON sur disque. Les 303 premières frames sont identiques ; le premier écart
est f303 : MutationChoice devient PendingMutations et le flux loot n'est plus
consommé immédiatement. Aucun autre composant ne change à cette frame.
`first-components.json` contient les valeurs exactes. Reproduction :

```sh
ALACOD_DUMP_TRACE=/tmp/m1-mutations cargo test -p scenario --profile headless \
  --test expectations throne_deferred_choice_trace_starts_at_the_earned_level -- --nocapture
python3 scripts/trace-diff.py /tmp/m1-mutations/immediate.full /tmp/m1-mutations/between-floors.full
```

Le code 1 de trace-diff et la première différence à f303 sont attendus.
Les changements de tirage à la fin du combat changent les mutations et les
combats suivants ; les attentes et traces des sept runs Floors sont mises à
jour après contrôle. Aucun profil de bot, arme ou difficulté n'est rééquilibré.

Timings observés après les deux correctifs de gameplay :

| Scénario | Nettoyage / choix / passage |
|---|---|
| throne_floor_1 | Portail ouvert f507 ; mutation et étage suivant f1108. |
| throne_mutation_choice | Dernier ennemi f539 ; ChoiceB choisit Vampire f784 ; passage f935. |
| throne_progression / solo | Étages suivants f1108 et f2805 ; Chasseur puis Vampire ; alerte de la caverne boss f3705. Solo vivant à f3799. |
| throne_softlock_recul | Portail étage 2 ouvert f1294 ; passage après choix f1895. |
| throne_three_floors (duo, graine 4) | Étages f874, f2036 et f4417 ; troisième étage terminé et duo vivant f4999 ; joueur 0 Adrénaline puis Coriace. |
| throne_quad | Étages f262, f1496 et f3162 ; troisième étage terminé et quatre joueurs vivants f3399. |

Les timings historiques des en-têtes sont explicitement signalés comme anciens.
Le scénario solo avait perdu à f2779 sous mutations Immediate après correction
du recul ; avec BetweenFloors il est vivant à la fin des 3800 frames contrôlées.
Cette différence est constatée, sans objectif de rééquilibrage implicite.
