# SB-5 — essai vocal dans le bac à sable : protocole et fiche à remplir

Statut au 2026-10-25 : **protocole écrit (SB-4), essai William à faire** ; toutes les cases de résultat sont « non joué ». Rien ci-dessous n'a été joué avec rendu : le VPS n'a pas d'affichage.
Ce qui a été **vérifié sans fenêtre** : les commandes `alacod-remote` de chaque ligne (essais headless de SB-3, `docs/taches/rapports/sb-3-mode-bac-a-sable.md`) et l'ouverture du bac à sable d'un jeu créé par `alacod new`
(SB-4, `docs/taches/rapports/sb-4-jeu-externe-docs.md`). Remplace la séance K5-4 (`docs/taches/rapports/k5-4-essai-vocal.md`, dont la décision du pont MCP reste valable et en attente).
Guide : `docs/editeur-vocal.md` (§2.2 phrases → commandes, §4 bloc à coller, §6 voix, §8 dépannage). Plan : `docs/taches/plan-bac-a-sable.md`.

## 1. Préparation, sur le Mac de William (une fois, avant la séance)

```bash
cd <dépôt alacod>                          # à jour (SB-1 à SB-4 mergés), git status vide
git checkout -b essai-bac-a-sable          # les commits « editor: » de la séance restent sur cette branche
APP_VERSION=dev cargo build -p zombies --features native    # première compilation longue ; rien à compiler pendant la séance
cargo run -q -p content --bin alacod -- lint games/zombies  # attendu : aucune erreur
say -v '?' | grep fr_FR                    # une voix française (Thomas, Amélie) ; sinon Réglages Système > Accessibilité > Contenu énoncé
```

Terminal A (fenêtre, journal visible) :

```bash
make sandbox GAME=zombies                  # arène de zombies, profil arene, mode éditeur ; remote sur 127.0.0.1:15702
```

Terminal B (même dépôt, à la racine) :

```bash
scripts/alacod-remote brief                # 1re ligne : editor=True online=False check_distance=0 … carte=exemples/test_map.ldtk graine=123456 sandbox=arene
ALACOD_VOICE=1 ALACOD_VOICE_NAME=Thomas claude     # voix macOS ; la dictée système est active dans ce terminal
```

Avant de commencer : coller le bloc « prêt à coller » de `docs/editeur-vocal.md` §4 dans le `CLAUDE.md` du dépôt **ou** le dire en première phrase. Couper la dictée pendant la lecture (boucle voix, §6). Chronomètre ou
horodatage à la main : on note le délai de la fin de la phrase à l'effet visible.

**À voir dès l'ouverture** (sinon : noter, ne pas corriger en séance) : la fenêtre s'ouvre sur l'arène, le joueur au centre-gauche, **aucun zombie, aucune vague** (`vague={'number': 0, 'phase': 'NotStarted'}` dans `brief`), étiquettes F6 visibles,
la même arène à chaque lancement (graine 123456). Le clavier joue (flèches ou ZQSD, tir à la souris).

Valeurs de départ à relire le jour même : profil `games/zombies/assets/sandbox/arene.ron` (solde `10000`, rien de posé) et `lourd.ron` (`machine_gun`, solde `50000`, `MoveSpeed ×1.2`, `zombie_1`, `zombie_2`, `zombie_full`
posés à droite) ; armes de `ZombieShooter/Sprites/Character/weapons.ron` ; ids d'ennemis : `alacod describe games/zombies Character`.

## 2. Déroulé

Dans chaque ligne, **l'attendu est la commande que Claude doit lancer** ; William ne tape rien. Une phrase après l'autre ; la partie ne se relance pas entre les lignes (sauf mention).

| # | William dit | Attendu (commande) | À voir | À entendre |
|---|---|---|---|---|
| 1 | « Quels profils ? » | `scripts/alacod-remote profile` | rien à l'écran ; Claude cite `arene` (actif) et `lourd` | une phrase |
| 2 | « Donne-moi le fusil. » | `scripts/alacod-remote give --weapon shotgun` (l'id vient de `weapons.ron` ; s'il hésite il liste les armes) | l'arme active devient le fusil à pompe ; `brief` : `arme shotgun`, munitions pleines | quoi, en une phrase |
| 3 | « Mets-moi 500 sous. » | `scripts/alacod-remote give --set-currency 500` | le solde du HUD passe de 10 000 à 500 (impose, n'ajoute pas) | une phrase |
| 4 | « Spawne-moi un zombie_1 devant moi. » (dictée : `zombie un`, `zombie_un`… Claude liste les ids et propose le plus proche) | `scripts/alacod-remote spawn zombie_1 --ahead` | un zombie apparaît ~60 px devant le joueur (dans sa direction de visée) et vient vers lui ; `brief` le liste | une phrase |
| 5 | « Un peu plus rapide. » | `scripts/alacod-remote modifier --handle 0 --source editor MoveSpeed Mul 1.1` (×1,1 à ×1,15) | le joueur va plus vite tout de suite, nettement mais pas du double | une phrase ; Claude dit que ça disparaît au `restart` |
| 6 | « Charge le profil lourd. » | `scripts/alacod-remote profile lourd` | la partie repart : `machine_gun`, trois zombies à droite, vitesse ×1,2 ; le fusil, le solde 500, le zombie et le ×1,1 des lignes 2 à 5 ont disparu ; `brief` : `sandbox=lourd` | une phrase |
| 7 | « Reset. » | `scripts/alacod-remote restart` | le profil `lourd` est rejoué : mêmes trois zombies, même arme ; ce que `give`/`spawn` ont ajouté depuis la ligne 6 disparaît | une phrase |
| 8 | « Redonne-moi le profil de départ. » | `scripts/alacod-remote profile arene` | arène vide, solde 10 000, aucun zombie | une phrase |
| 9 | **Édition** : « Le fusil fait un peu plus mal. » | édition de `shotgun…bullet_type…damage` (+10 à 15 %) dans `weapons.ron`, journal `armes rechargées`, lint vert, commit `editor: …` | dégâts flottants plus forts au prochain coup (les armes portées se rechargent à chaud) | quoi, où, quand |
| 10 | « Annule ça. » | `git log -1 --format=%s` commence par `editor:` puis `git revert --no-edit HEAD` | valeur revenue, journal `armes rechargées` | une phrase |
| 11 | **Ambiguë** : « C'est bizarre. » | Claude **pose une question** (quoi ? la vitesse, les dégâts, l'affichage ?) ; aucune commande, aucune édition | rien ne change | une question courte |
| 12 | **Refus** : relancer le bac à sable avec `ALACOD_CHECK_DISTANCE=2 make sandbox GAME=zombies`, puis « donne-moi le fusil » | `give` échoue ; Claude lit l'erreur et dit en une phrase que la cause est `check_distance=2` (relancer sans la variable) ; aucune édition silencieuse | rien ne change | la cause, dite |
| 13 | **Id inconnu** : « Donne-moi le lance-flammes. » | `give --weapon <id>` échoue (« arme inconnue ») ; Claude liste les armes du jeu et propose la plus proche, **sans** rien poser | rien ne change | la proposition |

Bonus si le temps le permet (jeu créé, sans code) : `alacod new demo --engine <dépôt alacod>` dans un dossier à part, puis `cd demo && cargo run --features native -- --sandbox --local-port 0 --players localhost` (première compilation
≈ 10 min) : arène du gabarit, joueur, cible à droite. Phrases 2 à 7 valent à l'identique, avec les armes du gabarit (`pistol`, `shotgun`, `machine_gun`), le solde `1000` de l'arène et la cible `target` (`spawn target --ahead`).
Un seul profil (`arene`) : « charge le profil lourd » doit répondre « inconnu » en listant `arene` ; `alacod new-content Sandbox lourd` en crée un.

Pour chaque ligne, noter : ce que William a dit mot pour mot (texte de la dictée), ce que Claude a fait (commande ou fichier édité), ce que William a vu, ce qu'il a entendu, le délai de la parole à l'effet, le verdict
(**marche / marche mal / ne marche pas**) et toute citation de William. Ne jamais compléter une case sans l'avoir observé.

Après la séance : `git log --oneline -8` (commits `editor:`), `git checkout <branche d'origine>`, jeter `essai-bac-a-sable` si on ne garde rien (décision de William), `git status` vide. Arrêt du jeu : fermer la fenêtre.

## 3. Fiche à remplir

| Phrase | Dit par William (dictée) | Fait par Claude | Vu | Entendu | Délai | Verdict |
|---|---|---|---|---|---|---|
| 1. « Quels profils ? » | non joué | non joué | non joué | non joué | non joué | non joué |
| 2. « Donne-moi le fusil. » | non joué | non joué | non joué | non joué | non joué | non joué |
| 3. « Mets-moi 500 sous. » | non joué | non joué | non joué | non joué | non joué | non joué |
| 4. « Spawne-moi un zombie_1 devant moi. » | non joué | non joué | non joué | non joué | non joué | non joué |
| 5. « Un peu plus rapide. » | non joué | non joué | non joué | non joué | non joué | non joué |
| 6. « Charge le profil lourd. » | non joué | non joué | non joué | non joué | non joué | non joué |
| 7. « Reset. » | non joué | non joué | non joué | non joué | non joué | non joué |
| 8. « Redonne-moi le profil de départ. » | non joué | non joué | non joué | non joué | non joué | non joué |
| 9. « Le fusil fait un peu plus mal. » | non joué | non joué | non joué | non joué | non joué | non joué |
| 10. « Annule ça. » | non joué | non joué | non joué | non joué | non joué | non joué |
| 11. Ambiguë (« C'est bizarre. ») | non joué | non joué | non joué | non joué | non joué | non joué |
| 12. Refus (`ALACOD_CHECK_DISTANCE=2`) | non joué | non joué | non joué | non joué | non joué | non joué |
| 13. Id inconnu (« le lance-flammes ») | non joué | non joué | non joué | non joué | non joué | non joué |
| Bonus. Jeu créé par `alacod new` | non joué | non joué | non joué | non joué | non joué | non joué |

Ouverture du bac à sable (§1 « À voir ») : non joué. Citations de William : non jouées. Problèmes rencontrés et correctif ou dette (à partir de la prochaine libre de `docs/taches/dettes.md`) : non joués.

## 4. Ce que la séance décide

- La **pose du rendu** (D94, D132) : les cinq jeux ouvrent-ils l'arène avec le profil à l'écran (joueur visible, étiquettes F6, caméra) ? Seule cette séance le voit.
- La boucle voix → commande → effet est-elle assez courte (délai noté) pour « donne-moi / mets-moi / charge / reset » ? Sinon : quelles commandes regrouper (`alacod-remote` fait déjà `profile` = chargement + `restart`).
- La décision du **pont MCP** reste celle de `k5-4-essai-vocal.md` §3 (le client vocal a-t-il un shell ?) : une seule réponse de William suffit pour les deux séances.
