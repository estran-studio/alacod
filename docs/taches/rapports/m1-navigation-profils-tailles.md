# Rapport m1-navigation-profils-tailles — navigation par profil et par taille (D41 + D38)

**Branche** `m1-navigation-profils-tailles`, partie de la tête livrée de D42 `780029f`. Fiche :
dix lignes validées par orch (garde-fous : profils des zombies vérifiés avant de coder ; un
commit = un lot de preuves ; bless à l'orchestrateur).

## État en cours

- **Fait** : garde-fou (a) vérifié (zombies : tous `GroundBreaker` sauf `zombie_scaled`, fixe :
  aucun champ lu) ; code des trois lots en WIP, non compilé (`469ce08` navigation par clé,
  `29e4fc5` mêlée pendant `Flee`, `a5dd973` boss 28 × 28).
- **En attente** : « poussé » d'orch (main avec intégration + D42), merge, compilation au feu vert.
