# Publication Cloudflare du site Alacod

Projet Pages existant : `alacod`, branche de production `main`.
Adresse de test : https://alacod.pages.dev ; domaine cible : `alacod.estran.studio`.
Le site utilise Allumette indépendant sur `https://allumette.estran.studio`.
Les deux origines publiques sont autorisées dans son CORS.

## Distribution des jeux

Les WASM de recette `rtc-fix-20261009` dépassent la limite Pages de 25 MiB.
Ils sont dans le bucket privé R2 `alacod-game-builds`, sous leurs clés immuables
`builds/<buildId>/<game>/wasm_bg.wasm`. Le reste des fichiers est servi par Pages.
La Pages Function `website/functions/builds/[[path]].js` lit seulement les WASM
Zombies/Throne et renvoie un stream de même origine : aucun téléchargement complet
n’est chargé dans la mémoire de la fonction. GET/HEAD, ETag/304 et ranges/206 sont
pris en charge. Le bucket ne nécessite ni domaine public ni CORS externe.
Voir la [recette officielle R2 + Pages](https://developers.cloudflare.com/pages/tutorials/use-r2-as-static-asset-storage-for-pages/).

`website/wrangler.jsonc` définit le binding `GAME_BUILDS` et le dossier de publication.
`node scripts/prepare-cloudflare.mjs` prépare `.cloudflare-dist` depuis `website/build`,
avec seulement les builds actifs du catalogue, et produit `.cloudflare-upload.json`
contenant tailles et SHA-256 des deux WASM à téléverser. Le staging et le manifeste
d’upload sont ignorés par Git. `_routes.json` limite les Functions aux URLs WASM;
le catalogue de releases n’est pas mis en cache.

## Refaire une publication

```sh
cd website
npx --yes wrangler@4.149.0 login
npx --yes wrangler@4.149.0 whoami
npm run check
npm run build
cd ..
node scripts/prepare-cloudflare.mjs
node --test website/tests/wasm-assets.test.mjs
```

Construire au préalable de nouveaux artefacts avec `scripts/build-web-games.mjs`.
Ne jamais remplacer le contenu d’un buildId déjà publié. Pour chaque nouvel objet
indiqué dans `.cloudflare-upload.json`, depuis `website` :

```sh
npx --yes wrangler@4.149.0 r2 object put \
  alacod-game-builds/builds/<buildId>/<game>/wasm_bg.wasm \
  --remote --file static/builds/<buildId>/<game>/wasm_bg.wasm \
  --content-type application/wasm \
  --cache-control 'public, max-age=31536000, immutable'
```

Après confirmation de tous les uploads, publier dans le même projet :

```sh
npx --yes wrangler@4.149.0 pages deploy .cloudflare-dist \
  --project-name alacod --branch main --commit-dirty=true
```

`--commit-dirty=true` décrit la publication depuis le travail local non commité;
omettre cette option pour un checkout propre. Ne pas publier un catalogue dont les
builds n’ont pas été livrés. Les workflows GitHub historiques doivent être alignés
sur cette distribution R2 avant de reprendre une publication automatique.

## Domaine personnalisé et exploitation

L’association Pages `alacod.estran.studio` a été créée via l’API Cloudflare, mais
l’OAuth Wrangler utilisé n’a pas les permissions DNS. Le record attendu est :
CNAME `alacod` → `alacod.pages.dev`, proxy activé. Pages gère ensuite le certificat.
L’adresse Pages canonique reste utilisable pendant cette activation. Les aliases
techniques de chaque déploiement ne sont pas inclus dans le CORS Allumette.

Conserver les objets R2 des anciens builds pour permettre un rollback Pages.
Dernier déploiement précédent : `09a84366-4b26-4150-bbbf-4d5e3f98b17c`.
Le rollback n’a pas été exercé. Vérifier les deux domaines, MIME WASM, ranges,
module/assets, catalogues et deux sessions avant de partager le lien.
Pour rejouer le test : `tools/web-smoke`, `SITE_ORIGIN=https://alacod.pages.dev`.
La recette sur deux contextes Linux ne remplace pas deux ordinateurs/réseaux d’amis.

## Déploiement courant — 2026-10-09

Production : `3d966a62-2801-4689-8ffb-d19f9d2d37b3`, créé à 21:30 UTC.
Wrangler 4.149.0 a publié 234 fichiers, la Function et son binding R2. Les deux
WASM ont été uploadés avant la disponibilité des jeux. Le build sélectionné est
`rtc-fix-20261009`, solo et online activés pour cette recette bêta demandée.
Le catalogue livré est enregistré dans `website/static/releases.json`.

Vérifiés : routes publiques 200, modules 200, WASM `application/wasm`, ranges 206
avec les huit octets attendus de l’en-tête WASM, ETag/304 et CORS Allumette pour
l’origine canonique. Quatre tests HTTP de la Function passent; compilation
Functions et Svelte passent. Le solo des deux clones passe sur le Mac depuis
l’adresse publique, sans erreur JS/HTTP et sans requête Allumette.

L’association du domaine personnalisé est en attente du record CNAME. Le jeton
OAuth fonctionne pour Pages/R2; l’API DNS refuse cet accès (403), donc aucune
modification DNS n’a été faite par cet outil.

Le parcours public multijoueur passe également pour **Zombies et Throne** depuis
deux contextes Chromium sur Linux : invitations/readiness, candidats TURN
`relay` des deux côtés, deux sessions successives et fermeture du salon. Aucune
erreur JavaScript, requête HTTP en échec ou désync observée. Les tests sont dans
`tools/web-smoke` (`test:online` et `test:solo`). Les deux appareils personnels
et les réseaux différents restent la prochaine recette humaine.
