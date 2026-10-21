# polissage-zombies-equilibrage-son — armes, son de vague et volumes de zombies (B6)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axes 4 et 5),
`docs/taches/m0-revue-suite-contenu.md:47-58` (S4, S6, S7), `docs/conventions.md` §16, §29 (dispersion), §15 (feedback),
`games/zombies/assets/ui/feedback.ron` (`volumes` existe déjà, `content/feedback.rs:63`), `weapons.ron` de zombies,
`docs/taches/dettes.md` (D32, D51). Branche `polissage-zombies-equilibrage-son` depuis `origin/main`. Estimation : 2 j.
Clone : zombies. Machine : VPS pour la campagne ; l'écoute des sons est de William (Mac).

## Contexte

Plaintes de William (revue M0, « gardées en file ») : fusil à pompe « super fort », pistolet inutile, mitraillette forte
mais trop imprécise (S4) ; tirs trop forts (S7) ; pas de son de vague (S6). Zombies a 4 armes, 6 configs, 1 boss, 6 salles
(inventaire §4) et **aucune campagne de bots chiffrée**.

## À faire

1. **Mesurer d'abord** : campagne `alacod-sim` sur zombies (un bot, deux bots, ≥ 100 graines : vagues atteintes, morts par
   cause, dégâts encaissés, armes utilisées) ; publier le tableau « avant ». Ne rien régler à l'aveugle.
2. Régler les valeurs de `weapons.ron` (dégâts, cadence, dispersion, prix) pour que les trois armes aient chacune un rôle ;
   changement de gameplay **voulu** : `make gen GAME=zombies`, traces rebénies (`BLESS=1`) avec **preuve §5** (première
   frame qui bouge et sa cause), à dire dans le commit. Pas d'arme ni d'effet nouveau.
3. S6 : son de début et de fin de vague **dérivé de l'état** (`WaveState` lu dans `Update`, jamais d'un événement de la
   simulation) ; échantillon déclaré dans `assets.yaml` (source, licence). Si aucun fichier libre n'est disponible, le
   dire et laisser à William plutôt que de créer un son.
4. S7 : rejouer les volumes de `feedback.ron` (les `volumes` valent aujourd'hui −12 dB sur la plupart : à juger à
   l'oreille avec William), ajuster ; présentation seule.
5. Campagne « après » et tableau comparé ; D32 : confirmer que zombies n'utilise pas l'échantillon soutenu.

## Acceptation

- Tableau de campagne avant/après dans `docs/taches/rapports/polissage-zombies-equilibrage-son.md`.
- Scénarios verts, traces rebénies avec preuve §5 ; sons : traces intactes (présentation).
- William a écouté : note « ni trop fort ni muet ».
