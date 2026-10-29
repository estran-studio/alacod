#!/usr/bin/env python3
"""Régularité du mouvement à l'écran (S1-6, `docs/conventions.md` §94), à partir d'un `ALACOD_PERF_CSV`.

    scripts/frame-pacing.py <perf.csv> [<perf.csv> ...]

Pour chaque fichier, sur les images où le joueur local se déplace (vitesse lissée sur ±3 images au-dessus
de 40 px/s, hors dash et téléport) :
- `hz` : fréquence de rendu (médiane des intervalles) ;
- `pas/image` : répartition du nombre de frames simulées par image (0, 1, 2, 3+) ;
- `figées` : part des images où le joueur ne bouge pas à l'écran alors qu'il se déplace ;
- `cv_pas` : écart type / moyenne du déplacement affiché image par image (0 = mouvement régulier ;
  l'écran présente les images à intervalle fixe, c'est la mesure de la saccade perçue) ;
- `cv_vitesse` : écart type / moyenne de la vitesse affichée image par image, rapportée à l'intervalle
  mesuré côté CPU (bruité par la mesure elle-même) ;
- `à-coups` : part des images en mouvement dont le pas affiché s'écarte de plus de 0,5 px de la moyenne
  des pas des deux images de part et d'autre (un pas de 0 ou de 5 px au milieu de pas de 2,5 px) ;
- `saccade_px` : écart type (px) de l'écart entre le déplacement affiché par image et le déplacement
  attendu à vitesse régulière (vitesse lissée × durée de l'image) ;
- `écran_px` : écart type (px) de la variation, d'une image à l'autre, de la position du joueur relative à
  la caméra (ce que l'œil suit : un joueur qui « tremble » dans un cadre qui glisse) ;
- `retard_ms` : retard moyen de l'image sur la dernière frame simulée, `(1 - alpha) × 16,7 ms` (0 sans
  interpolation).
"""

import csv
import statistics
import sys


def load(path):
    rows = []
    with open(path, newline="") as handle:
        for row in csv.DictReader(handle):
            if not row.get("player_x") or not row.get("camera_x"):
                continue
            rows.append(
                {
                    "frame": int(row["frame"]),
                    "dt": float(row["interval_ms"]) / 1000.0,
                    "px": float(row["player_x"]),
                    "py": float(row["player_y"]),
                    "cx": float(row["camera_x"]),
                    "cy": float(row["camera_y"]),
                    "alpha": float(row["alpha"]) if row.get("alpha") else None,
                }
            )
    return rows


def analyse(rows):
    if len(rows) < 10:
        return None
    steps = {0: 0, 1: 0, 2: 0, 3: 0}
    for prev, cur in zip(rows, rows[1:]):
        steps[min(max(cur["frame"] - prev["frame"], 0), 3)] += 1
    speeds, errors, screen, frozen, moving, displacements = [], [], [], 0, 0, []
    for i in range(4, len(rows) - 4):
        before, after = rows[i - 3], rows[i + 3]
        span = sum(r["dt"] for r in rows[i - 2 : i + 4])
        if span <= 0:
            continue
        smooth = ((after["px"] - before["px"]) ** 2 + (after["py"] - before["py"]) ** 2) ** 0.5 / span
        # Course : 150 px/s (sprint 300) ; au-delà, dash ou téléport, exclus.
        if smooth < 40.0 or smooth > 400.0:
            continue
        prev, cur = rows[i - 1], rows[i]
        if cur["dt"] <= 0:
            continue
        moving += 1
        step = ((cur["px"] - prev["px"]) ** 2 + (cur["py"] - prev["py"]) ** 2) ** 0.5
        if step < 0.01:
            frozen += 1
        displacements.append(step)
        speeds.append(step / cur["dt"])
        errors.append(step - smooth * cur["dt"])
        rel_prev = (prev["px"] - prev["cx"], prev["py"] - prev["cy"])
        rel_cur = (cur["px"] - cur["cx"], cur["py"] - cur["cy"])
        screen.append(((rel_cur[0] - rel_prev[0]) ** 2 + (rel_cur[1] - rel_prev[1]) ** 2) ** 0.5)
    jerks = 0
    for j in range(2, len(displacements) - 2):
        around = (displacements[j - 2] + displacements[j - 1] + displacements[j + 1] + displacements[j + 2]) / 4.0
        if abs(displacements[j] - around) > 0.5:
            jerks += 1
    intervals = sorted(r["dt"] for r in rows[1:] if r["dt"] > 0)
    hz = 1.0 / statistics.median(intervals) if intervals else 0.0
    total = sum(steps.values()) or 1
    mean_speed = statistics.mean(speeds) if speeds else 0.0
    mean_step = statistics.mean(displacements) if displacements else 0.0
    alphas = [r["alpha"] for r in rows if r["alpha"] is not None]
    interpolated = alphas and any(a > 0.0 for a in alphas)
    lag_ms = (1.0 - statistics.mean(alphas)) * 1000.0 / 60.0 if interpolated else 0.0
    return {
        "images": len(rows),
        "hz": hz,
        "steps": {k: steps[k] / total for k in steps},
        "moving": moving,
        "frozen": frozen / moving if moving else 0.0,
        "jerks": jerks / len(displacements) if displacements else 0.0,
        "cv_step": statistics.pstdev(displacements) / mean_step if mean_step else 0.0,
        "cv_speed": statistics.pstdev(speeds) / mean_speed if mean_speed else 0.0,
        "lag_ms": lag_ms,
        "judder_px": statistics.pstdev(errors) if errors else 0.0,
        "screen_px": statistics.pstdev(screen) if screen else 0.0,
    }


def main(paths):
    print("| fichier | images | hz | pas/image 0/1/2/3+ | images en mouvement | figées | à-coups | cv_pas | cv_vitesse | saccade_px | écran_px | retard_ms |")
    print("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for path in paths:
        result = analyse(load(path))
        if result is None:
            print(f"| {path} | trop peu d'images | | | | | | | | | | |")
            continue
        steps = "/".join(f"{100 * result['steps'][k]:.0f}%" for k in (0, 1, 2, 3))
        print(
            f"| {path} | {result['images']} | {result['hz']:.0f} | {steps} | {result['moving']} | "
            f"{100 * result['frozen']:.0f}% | {100 * result['jerks']:.0f}% | {result['cv_step']:.2f} | {result['cv_speed']:.2f} | "
            f"{result['judder_px']:.2f} | {result['screen_px']:.2f} | {result['lag_ms']:.1f} |"
        )


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    main(sys.argv[1:])
