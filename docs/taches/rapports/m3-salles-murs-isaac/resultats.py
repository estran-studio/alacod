"""usage : resultats.py <dossier de campagne> [préfixe…]   (défaut : s1 d2 q4)
m3-salles-murs-isaac : tableau des campagnes rejouées (JSON d'`alacod-sim`) face aux chiffres de T18 (TSV de
`docs/taches/rapports/m3-t18-integration-sortie/`, mêmes graines, mêmes réglages sauf `--explored-progress`)."""
import collections
import csv
import glob
import json
import os
import statistics
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
T18 = os.path.join(HERE, '..', 'm3-t18-integration-sortie')
out = sys.argv[1]
prefixes = sys.argv[2:] or ['s1', 'd2', 'q4']


def summary(rows):
    c = collections.Counter(r['issue'] for r in rows)
    wins = [r['frames'] for r in rows if r['issue'] == 'victoire']
    allf = [r['frames'] for r in rows]
    return {
        'graines': len(rows), 'victoires': c['victoire'], 'morts': c['mort'], 'soft-locks': c['soft-lock'],
        'plafonds': c['plafond'], 'desyncs': c['desync'] + sum(1 for r in rows if r.get('desync')),
        'étage 1 franchi': sum(1 for r in rows if r['etage'] >= 1),
        'frames victoire (médiane)': int(statistics.median(wins)) if wins else None,
        'frames partie (médiane)': int(statistics.median(allf)) if allf else None,
        'soft-lock graines': sorted(r['seed'] for r in rows if r['issue'] == 'soft-lock'),
        'plafond graines': sorted(r['seed'] for r in rows if r['issue'] == 'plafond'),
    }


for p in prefixes:
    before = [{'seed': int(r['graine']), 'issue': r['issue'], 'frames': int(r['frames']), 'etage': int(r['etage']), 'desync': False}
              for r in csv.DictReader(open(f'{T18}/{p}.tsv'), delimiter='\t')]
    after = []
    for path in sorted(glob.glob(f'{out}/{p}_*.json')):
        if path.endswith('.tmp.json'):
            continue
        for d in json.load(open(path)):
            after.append({'seed': d['seed'], 'issue': d['outcome'], 'frames': d['frames'], 'etage': d['floor'], 'desync': d.get('desync', False)})
    seeds_after = {r['seed'] for r in after}
    print(f'== {p}: avant (T18) {len(before)} graines, après {len(after)} graines')
    for name, rows in (('avant', before), ('après', after)):
        print(f'  {name:6}', summary(rows))
    both = [r for r in before if r['seed'] in seeds_after]
    if len(both) != len(before):
        print('  avant, restreint aux graines déjà jouées après :', summary(both))
