# Recette multijoueur — 2026-10-09

Build `rtc-fix-20261009`, deux Chromium 154 isolés sur Debian 13, renderer logiciel.
Le site de recette est servi sur localhost; le signaling utilise le VPS Allumette
HTTPS/WSS et les deux côtés passent par TURN forcé. Les captures montrent le
premier gameplay après quelques entrées clavier. Le test rejoue ensuite une
seconde session et ferme le salon sans erreur JavaScript/HTTP ni désync observée.

- [Zombies à deux](zombies-turn.png)
- [Throne à deux](throne-turn.png)

Ces captures accompagnent les vérifications RTC et engine; une image seule ne
prouve pas la connexion. Ce test sur un seul hôte ne mesure pas les performances
chez les amis et ne remplace pas la recette sur deux machines/réseaux.
