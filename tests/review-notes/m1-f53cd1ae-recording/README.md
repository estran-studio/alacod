# Enregistreur natif — correctif M1

Autorisé le 2026-10-10 avec le plan des correctifs du playthrough.

L'enregistreur natif prend maintenant le nom du manifeste et la configuration effective
au **premier input GGRS**, avant de l'enregistrer : mode, séquence de niveaux, horloges,
difficulté et progression. Il conserve la carte et la graine initiales. Les scénarios
conservent leurs réglages explicites et leurs paramètres par joueur.

Un restart écrit d'abord la run précédente, puis vide le buffer et utilise un fichier
`<nom>-r2.ron`, puis `-r3.ron`, etc. Si l'archivage échoue, le buffer précédent est
préservé et l'enregistrement est suspendu avec un log d'erreur. La fermeture écrit la
run courante. La limite historique reste : seuls les inputs des joueurs locaux sont
capturés ; ce n'est pas une preuve de rejeu d'une session p2p complète.

Validation :

- `throne_native_recording_roundtrip` : vraie configuration de démarrage throne,
  400 frames d'inputs de bot capturées avec l'enregistreur **natif**, sérialisation RON,
  rejeu scripté ; **toutes les traces identiques**, réglages et graine contrôlés.
- `native_restart_archives_previous_inputs_and_resets_the_frame_range` : système de
  début de session, fichier archivé relu, nouvelle plage vide et chemin `-r2.ron`.
- Le test historique des réglages de scénario reste vert.

Aucun décalage ajouté aux inputs : le test compare la trace frame par frame. Le fichier
humain original reste intact dans [run 01](../m1-f53cd1ae-run-01/README.md).

Smoke natif Mac rendu : `ALACOD_RESTART_AT_FRAME=30`, puis sortie à f60
dans la seconde partie, avec `ALACOD_STATE_TRACE` et `ALACOD_RECORD`. Les deux
fichiers RON conservent bien throne/Floors/run/etage/difficulté/graine 123456.
L'ancienne session continue jusqu'à sa fermeture effective (92 inputs), puis
la seconde enregistre 60 inputs distincts. Les exports GGRS contiennent 29 et
59 lignes, conformément à la dernière sauvegarde manquante documentée dans
state_trace.rs. Les quatre fichiers sont joints ; aucune validation humaine
du bouton restart n'est déduite de ce smoke automatique.
