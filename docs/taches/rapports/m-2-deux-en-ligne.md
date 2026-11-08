# M-2 — Salon Chrome et lanceur public d'un jeu externe

Jalon servi : **S1 dim 15 novembre**, cible **ven 13**, « 1837 à deux en ligne dans Chrome ».
État au 8 novembre 2026 : **page, serveur et lanceur livrés ; partie à deux non recettée**.

Le coordinateur a explicitement autorisé la remise de l'engine et du lanceur avec les preuves
locale/publique laissées à William après l'échec derrière le VPN d'entreprise. Cette remise ne
prouve ni une partie commune ni l'absence de desync en partie. Aucun VPN ni réglage réseau système
n'a été changé.

## Résultat par point

| Point demandé | Issue | Preuve |
|---|---|---|
| Relevé avant code, choix de signalisation | fait | status Orca `msg_4e1f869398fe`, dix lignes ; choix confirmé : Matchbox local + tunnels cloudflared à la demande |
| Page générique par query string | fait | engine **`4c54026d1f424b4604e9d10918cb10a45c8a7ef2`**, `crates/content/src/web_build.rs::index_html` ; les deux Chrome lisaient le même `data-lobby`, `data-matchbox` et `data-number-player="2"` |
| Serveur wasm compressé | fait | build produit le `.wasm.gz` ; HEAD réel renvoie `application/wasm`, `Content-Encoding: gzip`, `Vary: Accept-Encoding` et la taille compressée |
| `./jouer.sh --web <salon> [--public]` | fait | branche 1837 **`m-2-en-ligne`**, code **`7dad844645ef317aaccd5f56eb0a998f9a3b11b5`** ; lanceur local réel construit et sert le jeu épinglé, arrêt vérifié par Ctrl-C |
| Tunnel hors réseau local | implémenté, accès public non vérifié | cloudflared démarre, mais QUIC puis HTTP/2 échouent sur la sortie d'entreprise ; l'essai du vrai `--public` sort code 1, sans annoncer une URL prête |
| Deux Chrome, même salon, partie sans desync | **non prouvé** | deux fenêtres muettes chargent le jeu et le signaling, restent « En attente des joueurs » ; ICE `failed`, aucune paire sélectionnée |
| Protocole pour William et son ami | fait | dix lignes ci-dessous et `1837/README.md` ; poste/réseau personnel sans VPN d'entreprise requis pour la recette |
| Contrôle rapide web | **vert** | tête engine poussée `4c54026d`, journal complet `~/orca/runs/alpha/logs/m-2-deux-en-ligne-check-rapide-4c54026d.log` |
| Documentation et dettes | fait | `docs/conventions.md` §109, D192–D193 dans `docs/taches/dettes.md`, état dans `1837/docs/tranche.md`, ligne LIVRÉ |

## Choix et frontière

- `?lobby=` active Matchbox ; `?joueurs=` donne le nombre de joueurs (défaut 2).
  `?matchbox=` donne la base WS/WSS ; absent, hôte de la page et port 3536, WSS en HTTPS.
  Sans salon, la page reste solo. Le salon est encodé comme un segment de chemin.
- Le lecteur de canvas existant (`crates/game/src/args/web.rs`) suffisait : **inchangé**.
  Aucun fichier de `crates/game/src/jjrs`, système de `GgrsSchedule`, type rollback, contenu
  de simulation ou scénario modifié. Aucun `BLESS`, aucune comparaison de traces prétendue.
- `polissage-reseau-amis` @ `8297b2ea` **non fusionnée** : admission bornée, retour au salon
  et travail de ticket Allumette préparés, mais recette entre deux réseaux non tenue.
  Ce contrat Allumette (session/JWT) n'est pas celui du Matchbox nu retenu ici.
- Les tunnels rendent publics page et signaling, pas les paquets de jeu : **STUN existant,
  aucun TURN ajouté** (D192). Pas de compte, brew, service ou déploiement permanent.
- Le lanceur télécharge cloudflared **2026.10.0** dans `target/outils/`, vérifie le SHA256
  de l'archive macOS arm64 (`a2f79ff7b9420aa537d74af239f376da170bbabeb529aec416002adac6a72e70`)
  ou x86_64, utilise HTTP/2, attend `Registered tunnel connection` avant d'annoncer l'URL.
  Un enfant fermé fait échouer cette attente ; le lanceur ferme ses PID exacts à la sortie.
- D193 garde la dépendance au Mac et à sa sortie réseau. Si cela empêche S1 : lot distinct
  autorisé d'hébergement et de signaling WSS sur 443 administré (par exemple VPS du studio),
  pas une modification opportuniste du réseau du jeu dans M-2.

## Vérifications exercées

### Build, HTTP et interruption du vrai lanceur

1. Build web de référence : `alacod build --web --release --version 41861acf80da` avec le CLI
   du worktree, 150 assets et wasm optimisé ; page réellement ouverte dans Chrome.
2. Après épinglage des sept dépendances et de `Cargo.lock` sur `4c54026d`, le vrai lanceur
   `jouer.sh --web m2-final` a reconstruit le jeu **`7dad844645ef`** en **2 min 10 s**,
   puis servi la page sur `127.0.0.1:8080` et Matchbox sur `127.0.0.1:3536`.
   Pour employer le CLI local de debug, la preuve utilisait `ALACOD_BIN`, bash Homebrew et
   le `DYLD_LIBRARY_PATH` de `scripts/dev-features.sh libpath` (le premier lancement sans ce
   chemin échouait au chargement de `libstd`, particularité du binaire de contrôle dynamique).
   Le lanceur normal installe son CLI release du `rev`, sans cet override de travail.
3. `curl --head -H 'Accept-Encoding: gzip' http://127.0.0.1:8080/builds/7dad844645ef/1837/wasm_bg.wasm` :
   HTTP 200, `Content-Type: application/wasm`, `Content-Length: 21183815`, `Content-Encoding: gzip`,
   `Vary: Accept-Encoding`.
4. Ctrl-C : sortie 130, puis vérification socket : **8080 fermé, 3536 fermé**.
5. Le vrai `jouer.sh --web m2-public --public` a lancé les deux cloudflared HTTP/2, détecté
   leur échec et quitté **code 1** ; journaux complets dans les preuves du jeu externe.
6. `bash -n jouer.sh scripts/jouer-web.sh` vert ; option inconnue refusée avec le code 2.

| Artefact du build épinglé | Octets |
|---|---:|
| `wasm_bg.wasm` | 86 002 809 |
| `wasm_bg.wasm.gz` | 21 183 815 |

### Chrome et réseau : limite de la preuve

Deux vraies fenêtres Chrome, muettes, même URL et même salon ; pas de boucle de navigateur,
aucune campagne de bots. Le signaling observe les deux clients et `NewPeer`. Les deux écrans
restent en attente ; les offres/réponses existent, mais ICE finit `failed` sans paire sélectionnée.
Le diagnostic initial montre des candidats host mDNS et srflx `165.225.213.5`, puis un timeout STUN 701.

Le coordinateur a autorisé **un seul essai supplémentaire**, sur un profil temporaire dédié,
avec `--disable-features=WebRtcHideLocalIpsWithMdns` et `--mute-audio` : même échec. Ce drapeau
ne sert qu'au diagnostic du poste derrière VPN, **les joueurs ne l'ont pas**, il n'est pas dans
le lanceur. Chrome a été fermé par son PID ; le profil temporaire supprimé.

cloudflared : QUIC/UDP 7844 sans réponse, puis HTTP/2/TLS 7844 en timeout ou connexion réinitialisée,
source `100.64.0.1`, interface `utun4`. Les précontrôles réussissent DNS et API mais pas la connexion
finale du tunnel ; **une URL allouée n'est pas une URL connectée**. Aucun contournement du VPN.

Preuves conservées dans le dépôt privé 1837, `docs/preuves/m-2/` :
- `local-vpn-a.png`, `local-vpn-b.png` : les deux écrans en attente, **pas une partie à deux** ;
- `webrtc-vpn.png`, `ice-vpn.json` : ICE en échec, diagnostic sans identifiants ICE persistés ;
- `tunnel-vpn.log` : extraits des essais QUIC/HTTP2 ;
- `public-page-vpn.log`, `public-signal-vpn.log` : journaux complets du vrai lanceur public.

### Tests et contrôle rapide

`cargo test -p content --profile headless --lib web_build::tests` : **4 passés**.
Le test serveur couvre le MIME wasm, le repli brut, gzip accepté/refusé (`q=0`), HEAD et l'interdiction de `..`.
La preuve de surface est le vrai serveur et Chrome ci-dessus, pas les seuls tests.

```text
$ env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 make check_rapide WASM=1
    Finished `dev` profile [optimized + debuginfo] target(s) in 3m 06s
✓ Contrôle rapide passé (suite de scénarios : la nuit)
```

Journal complet : `~/orca/runs/alpha/logs/m-2-deux-en-ligne-check-rapide-4c54026d.log`
(contrôle total 1 486 s ; dernière compilation wasm 3 min 06 s). Ensuite, les sweeps requis
`target` et `target_wasm` ont été joués, sans build concurrent. Le commit final de rapport et
les précisions de D193 sont documentaires seulement ; le code contrôlé reste `4c54026d`.

## Protocole du soir — dix lignes

1. William : poste/réseau personnel sans VPN d'entreprise, manette branchée, terminal dans 1837.
2. Lancer `./jouer.sh --web soir-13 --public` ; attendre les tunnels connectés puis la fin du build et le serveur local prêt.
3. Envoyer l'URL HTTPS complète affichée (`lobby`, `matchbox=wss://…`, `joueurs=2`) à l'ami ; lui n'installe rien.
4. L'ami ouvre ce lien dans Chrome sur un autre réseau (ou partage téléphone), manette branchée.
5. William ouvre le même lien ; chacun clique dans son canvas et appuie sur un bouton de manette.
6. Attendre la salle à deux ; relever le délai, vérifier que chacun voit l'autre, ne pas ouvrir un troisième onglet du salon.
7. Sticks : déplacement/visée ; RT : fusil ; B : hache ; X : recharge ; LB : esquive ; jouer cinq minutes.
8. Aucun écran DESYNC ne doit apparaître ; garder une capture de chaque écran et les erreurs de console éventuelles.
9. Si bloqué : noter VPN/NAT, candidat sélectionné de `chrome://webrtc-internals`, état des tunnels et journaux `target/outils/` ; D192–D193.
10. Fermer les deux onglets puis Ctrl-C chez William ; noter le résultat au coordinateur : cette partie fait foi.

## Remise

Engine : `wquintal/m-2-deux-en-ligne` ; jeu : `m-2-en-ligne` sur le dépôt 1837,
tête poussée **`985d7758220cdb62f0c29017d71af57bf9292596`** (les derniers changements sont les preuves et la documentation).
Les sept rev et le verrou du jeu visent le SHA engine poussé `4c54026d` avec l'accord du coordinateur ;
après merge engine, **le coordinateur 1837** réépingle sur `main` en intégrant la branche.
Aucun merge de `main`, aucun déploiement, aucun changement système effectué par le worker.
Reste : recette locale et publique sur réseau sans VPN, manette réelle et partie commune sans desync.
