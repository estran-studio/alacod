"""usage : choisir_graines.py <dossier des événements> <préfixe>   (ex. /tmp/b6-camp/events s1)
m3-salles-murs-isaac : comme T18 (`m3-t18-integration-sortie.md` §1), la plus petite graine qui réunit les moments demandés par un scénario
de run complète, lue dans les moments clés de `alacod-sim --events` (une partie de bot, séquence par défaut). Les prédicats suivent la liste de
`isaac_full_run` (critère 3 de T18) et de `isaac_curse_run` ; ils nomment des **mécaniques**, jamais un objet précis (l'objet vendu dépend de la graine)."""
import glob
import json
import os
import re
import sys

root = os.path.dirname(os.path.abspath(__file__)) + '/../../../..'
objets = {}
for path in glob.glob(f'{root}/games/isaac/assets/objets/*.ron'):
    text = re.sub(r'//[^\n]*', '', open(path).read())
    tags = re.search(r'tags:\s*\[([^\]]*)\]', text)
    objets[os.path.basename(path)[:-4]] = {'larmes': bool(tags and '"larmes"' in tags[1]), 'familier': 'familiar:' in text}


def load(folder, prefix):
    out = {}
    for path in glob.glob(f'{folder}/{prefix}/seed_*.events.json'):
        out[int(re.search(r'seed_(\d+)', path)[1])] = json.load(open(path))
    return dict(sorted(out.items()))


def first(ev, kind, part='', after=-1):
    for e in sorted(ev, key=lambda e: e['frame']):
        if e['kind'] == kind and part in e['label'] and e['frame'] > after:
            return e['frame']
    return None


def checks_full(ev):
    portal0 = first(ev, 'portal')
    sold = [(e['frame'], re.search(r'vendu : (\S+)', e['label'])[1]) for e in ev if e['kind'] == 'shelf' and 'vendu' in e['label']]
    floor1_sold = [s for s in sold if portal0 is None or s[0] < portal0]
    floor2_sold = [s for s in sold if portal0 is not None and s[0] > portal0]
    bomb = first(ev, 'bomb')
    secret0 = first(ev, 'door_lock', 'mur secret ouvert') if bomb else None
    rock = first(ev, 'feedback', 'rocher cassé', after=(secret0 or 10**9) - 40)
    deaths2 = [e['label'] for e in ev if e['kind'] == 'variant_death' and portal0 is not None and e['frame'] > portal0]
    names = {re.search(r'variante (\S+)', d)[1] for d in deaths2 if re.search(r'variante (\S+)', d)}
    return {
        'combat verrouillé puis nettoyé': first(ev, 'room', 'salle combat verrouillée') and first(ev, 'room', 'salle combat nettoyée'),
        '≥ 3 achats en sous au premier étage, dont un objet de larmes': len(floor1_sold) >= 3 and any(objets.get(i, {}).get('larmes') for _, i in floor1_sold),
        'cœur ramassé': any(e['kind'] == 'item_pickup' and re.search(r'ramasse (coeur|coeur_ame|pain|festin|eclat_ame)$', e['label']) for e in ev),
        'bombe : mur secret ouvert (niveau 0) et rocher cassé par la même explosion': bool(secret0 and rock and rock <= secret0 + 40),
        'boss : verrouillé, phases 0→1 et 1→2, portail': bool(first(ev, 'room', 'salle boss verrouillée') and first(ev, 'boss_phase', 'phase 0 → 1') and first(ev, 'boss_phase', 'phase 1 → 2') and portal0),
        'étage 2 : deux variantes de champion tuées': len(names) >= 2,
        'porte à clé ouverte (niveau 1)': bool(first(ev, 'door_lock', 'porte à clé ouverte')),
        'second mur secret (niveau 1)': bool(first(ev, 'door_lock', 'mur secret ouvert : salle secrete (niveau 1)')),
        'familier acheté au second étage': any(objets.get(i, {}).get('familier') for _, i in floor2_sold),
        'second boss : phases': bool(first(ev, 'boss_phase', 'phase 1 → 2', after=(portal0 or 10**9))),
        'victoire': bool(first(ev, 'victory')),
    }


def checks_curse(ev):
    portal0 = first(ev, 'portal')
    return {
        'curse darkness au premier étage': any(e['kind'] == 'curse' and 'étage 1' in e['label'] and 'darkness' in e['label'] for e in ev),
        "pas de curse au second étage": not any(e['kind'] == 'curse' and 'étage 2' in e['label'] for e in ev),
        'mur secret ouvert (niveau 0)': bool(first(ev, 'door_lock', 'mur secret ouvert : salle secrete (niveau 0)')),
        'boss : phases 0→1 et 1→2, portail': bool(first(ev, 'boss_phase', 'phase 0 → 1') and first(ev, 'boss_phase', 'phase 1 → 2') and portal0),
        'second étage atteint': bool(first(ev, 'floor')),
    }


if __name__ == '__main__':
    folder, prefix = sys.argv[1:3]
    data = load(folder, prefix)
    for name, fn in (('isaac_full_run', checks_full), ('isaac_curse_run', checks_curse)):
        rows = {s: fn(ev) for s, ev in data.items()}
        ok = [s for s, r in rows.items() if all(r.values())]
        print(f'{name} : {len(data)} graines lues ; toutes les conditions : {ok[:15]}')
        miss = sorted(((sum(1 for v in r.values() if not v), s, [k for k, v in r.items() if not v]) for s, r in rows.items()))[:6]
        for n, s, m in miss:
            if n:
                print(f'   graine {s} : {n} manquant(s) : {m}')
