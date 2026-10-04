# Rapport m1-v4-feedback-v1 — feedback v1 : hit stop, secousse, flash, télégraphe, chiffres (T1.17)

**Branche** `m1-v4-feedback-v1`, partie de la tête livrée de m1-throne-gen-et-d40 `226e8aa`.
Fiche : `docs/taches/m1-v4-feedback-v1.md`.

## État en cours

- **Fait** : branche créée, target purgé (fin de la tâche précédente : 38G, /home 93G libres),
  fiche lue, décisions confirmées à orch en dix lignes (amendements : hit stop en frames de rendu
  sans toucher `Time<Virtual>` ; arme résolue depuis la source, `DamageEvent` étant tracé ; flash
  blanc actuel invisible ; `FeedbackPlugin` scindé pour remplir `FeedbackLog` en headless).
- **En cours** : code, sans compiler jusqu'au feu vert d'orch.
