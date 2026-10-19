"""usage : couverture.py <dossier> <préfixe>…   (ex. /tmp/m3-t18/camp s1 d2 q4)
M3-T18 : tableau des campagnes (critère 4 du §8.1) et couverture du tableau « Atteignabilité » (part des graines où la
mécanique est rencontrée), lus dans les JSON d'`alacod-sim`, les moments clés `--events` et les morts de `flux.py`.
Objets classés par leur RON (`games/isaac/assets/objets`) : tag `larmes`, champ `familiar`, champ `effects` (hors consommables)."""
import collections
import glob
import json
import os
import re
import statistics
import sys

root = os.path.dirname(os.path.abspath(__file__)) + '/../../../..'
objets = {}
for path in glob.glob(f'{root}/games/isaac/assets/objets/*.ron'):
    text = re.sub(r'//[^\n]*', '', open(path).read())
    tags = re.search(r'tags:\s*\[([^\]]*)\]', text)
    objets[os.path.basename(path)[:-4]] = {
        'larmes': bool(tags and '"larmes"' in tags[1]),
        'familier': 'familiar:' in text,
        'effets': 'effects:' in text and 'Consumable' not in text,
        'consommable': 'Consumable' in text,
    }
COEURS = {'coeur', 'coeur_ame', 'pain', 'festin', 'eclat_ame'}


def rows(events, devil_floors):
    picks = collections.defaultdict(list)  # joueur -> objets
    for e in events:
        if e['kind'] == 'item_pickup':
            m = re.match(r'joueur (\d+) ramasse (\S+)', e['label'])
            if m:
                picks[m[1]].append(m[2])
    every = [i for p in picks.values() for i in p]
    larmes = {h: [i for i in p if objets.get(i, {}).get('larmes')] for h, p in picks.items()}
    has = lambda kind, start='': any(e['kind'] == kind and e['label'].startswith(start) for e in events)
    contains = lambda kind, part: any(e['kind'] == kind and part in e['label'] for e in events)
    return {
        'objet de larmes ramassé': any(larmes.values()),
        'empilement : deux objets de larmes sur un joueur': any(len(v) >= 2 for v in larmes.values()),
        'deux exemplaires du même objet sur un joueur': any(len(p) != len(set(p)) for p in picks.values()),
        'objet à effets ramassé': any(objets.get(i, {}).get('effets') for i in every),
        'transformation': has('feedback', 'transformation'),
        'familier ou orbital ramassé': any(objets.get(i, {}).get('familier') for i in every),
        'bombe du joueur qui explose (rayon 56)': contains('explosion', 'rayon 56'),
        'rocher cassé': has('feedback', 'rocher cassé'),
        'salle secrète ouverte (mur secret)': contains('door_lock', 'mur secret'),
        'porte à clé ouverte (trésor fermé)': contains('door_lock', 'porte à clé'),
        'cœur ramassé (coeur, coeur_ame, pain, festin, eclat_ame)': any(i in COEURS for i in every),
        'sous ramassés': contains('points', '(pickup)'),
        'achat en boutique (sous)': contains('shelf', 'douilles'),
        'objet tiré d\'un pool et pris (coffre, présentoir)': has('chest') or has('shelf'),
        'salle du diable générée sur un étage atteint': bool(devil_floors),
        'achat au diable': contains('shelf', 'diable'),
        'champion apparu': has('variant_spawn'),
        'champion tué': has('variant_death'),
        'curse': has('curse'),
        'boss et phases': has('boss_phase'),
        'portail ouvert (premier boss tué)': has('portal'),
    }


def main():
    d = sys.argv[1]
    # Dernières chutes rejouées par `chutes.py` (bots, graine, joueur, frame, source, tags), quand elles existent.
    chutes = collections.defaultdict(list)
    if os.path.exists(f'{d}/chutes.tsv'):
        for line in open(f'{d}/chutes.tsv'):
            c = line.rstrip('\n').split('\t')
            if len(c) >= 5:
                chutes[(c[0], int(c[1]))].append(c)
    for p in sys.argv[2:]:
        R = []
        for f in sorted(glob.glob(f'{d}/{p}_*.json')):
            if not f.endswith('.tmp.json'):
                R += json.load(open(f))
        morts, devil = collections.defaultdict(list), {}
        for f in sorted(glob.glob(f'{d}/{p}_*.morts.tsv')):
            for line in open(f):
                c = line.rstrip('\n').split('\t')
                if c[1] == 'diable':
                    devil[int(c[0])] = [] if c[2] == '-' else c[2].split(',')
                else:
                    morts[int(c[0])].append(c)
        n = len(R)
        if not n:
            continue
        R.sort(key=lambda r: r['seed'])
        bots = p[1:]
        oc = collections.Counter(r['outcome'] for r in R)
        vic = [r['frames'] for r in R if r['outcome'] == 'victoire']
        crossed = sum(r['floor'] >= 1 for r in R)
        print(f'## {p} : {n} graines')
        print(f'issues {dict(oc)} ; desync {sum(r["desync"] for r in R)} ; étage 1 franchi {crossed} ({100 * crossed / n:.1f} %) ;'
              f' victoires {oc["victoire"]} ({100 * oc["victoire"] / n:.1f} %) ; frames de victoire médiane {int(statistics.median(vic)) if vic else "-"}')
        print('soft-locks', [r['seed'] for r in R if r['outcome'] == 'soft-lock'], 'plafonds', [r['seed'] for r in R if r['outcome'] == 'plafond'])
        # Cause d'une partie perdue : la dernière chute de chaque joueur (`chutes.py`) si la graine a été rejouée, sinon le
        # tueur du journal de la campagne (`flux.py`, première mort).
        causes, floors = collections.Counter(), collections.Counter()
        for r in R:
            if r['outcome'] != 'mort':
                continue
            floors[r['floor']] += 1
            replayed = chutes.get((bots, r['seed']))
            if replayed:
                for c in replayed:
                    causes[c[4]] += 1
            else:
                for c in morts.get(r['seed'], []):
                    causes[c[5]] += 1
        print(f'parties perdues {oc["mort"]} ; par étage (0 = premier) {dict(sorted(floors.items()))} ;'
              f' causes (dernière chute de chaque joueur) {causes.most_common()}')
        cov = collections.Counter()
        seeds = collections.defaultdict(list)
        lines = []
        for r in R:
            path = f'{d}/events/{p}/seed_{r["seed"]}.events.json'
            events = json.load(open(path)) if os.path.exists(path) else []
            flags = rows(events, devil.get(r['seed'], []))
            for k, v in flags.items():
                cov[k] += v
                if v:
                    seeds[k].append(r['seed'])
            lines.append('\t'.join([bots, str(r['seed']), r['outcome'], str(r['frames']), str(r['floor'])] + [str(int(v)) for v in flags.values()]))
        print('| Mécanique | graines | part | ex. |')
        print('|---|---|---|---|')
        for k in rows([], []):
            print(f'| {k} | {cov[k]} / {n} | {100 * cov[k] / n:.0f} % | {" ".join(map(str, seeds[k][:6]))} |')
        print()
        out = os.environ.get('TSV_DIR')
        if out:
            with open(f'{out}/{p}.tsv', 'w') as f:
                f.write('\t'.join(['bots', 'graine', 'issue', 'frames', 'etage'] + list(rows([], []))) + '\n')
                f.write('\n'.join(lines) + '\n')


main()
