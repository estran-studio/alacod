"""Régressions des garde-fous : motifs auparavant invisibles, exceptions périmées,
mesures manquantes et faux succès de comparaison. Exécuter `make test_tools`.
"""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]


def module(name):
    spec = importlib.util.spec_from_file_location(name, SCRIPTS / f"{name}.py")
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


guard = module("check-forbidden")
perf = module("perf-review")
traces = module("check-p2p-traces")


class DeterminismGuardTests(unittest.TestCase):
    def test_grouped_bevy_import_and_unqualified_usage_are_detected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for crate in guard.CRATES:
                (root / "crates" / crate / "src").mkdir(parents=True)
            (root / "crates/new_sim/src").mkdir(parents=True)
            source = root / "crates/new_sim/src/new.rs"
            source.write_text("// HashMap et rand:: dans la documentation\n"
                              "use bevy::platform::collections::{HashMap, HashSet};\n"
                              "let cache = HashMap::new();\n"
                              "let cache = HashMap::new();\n"
                              "rand::random::<u32>();\n")
            found, _ = guard.findings(root)
            self.assertEqual(sum(found.values()), 4)
            self.assertEqual(found[("crates/new_sim/src/new.rs", "let cache = HashMap::new();")], 2)

    def test_missing_source_directory_is_not_a_success(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(ValueError):
                guard.findings(Path(directory))

    def test_exception_must_be_unique_and_justified(self):
        entry = dict(path="x", code="HashMap", count=1, reason="")
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "allow.json"
            path.write_text(json.dumps([entry]))
            with self.assertRaises(ValueError):
                guard.load_allowances(path)
            entry["reason"] = "lookup only"
            path.write_text(json.dumps([entry, entry]))
            with self.assertRaises(ValueError):
                guard.load_allowances(path)

    def test_current_repository_has_no_unreviewed_or_stale_exception(self):
        found, _ = guard.findings(SCRIPTS.parent)
        allowed = guard.load_allowances(SCRIPTS / "determinism-allowlist.json")
        self.assertEqual(found, allowed)


class PerformanceReviewTests(unittest.TestCase):
    def metrics(self, fps=180, frames=600, players=4, **costs):
        return {"bench": dict(sim_fps=fps, frames=frames, players=players, **costs)}

    def compare(self, current, baseline, names=("bench",), minimum=None, max_rollback=None):
        return perf.compare(current, baseline, names, 10, minimum, max_rollback)[1]

    def test_regression_is_blocking_even_above_absolute_floor(self):
        self.assertTrue(self.compare(self.metrics(160), self.metrics(200)))

    def test_throughput_objective_is_independent_of_non_regression(self):
        self.assertTrue(self.compare(self.metrics(50), self.metrics(50), minimum=180))

    def test_missing_scenario_cannot_pass(self):
        self.assertTrue(self.compare({}, self.metrics()))

    def test_different_workloads_cannot_pass(self):
        self.assertTrue(self.compare(self.metrics(frames=300), self.metrics()))
        self.assertTrue(self.compare(self.metrics(players=2), self.metrics()))

    def test_improvement_passes(self):
        self.assertEqual(self.compare(self.metrics(220), self.metrics(200), minimum=180), [])

    def test_invalid_measurements_fail_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "metrics.json"
            for value in (0, -1, float("nan"), float("inf"), True, "180"):
                path.write_text(json.dumps(self.metrics(value)))
                with self.assertRaises(ValueError):
                    perf.load_metrics(path)
            path.write_text("[]")
            with self.assertRaises(ValueError):
                perf.load_metrics(path)

    def test_frame_cost_increase_is_a_regression_even_with_same_fps(self):
        # La frame de simulation coûte 20 % de plus : le débit du harness peut masquer le
        # surcoût (moins de rollbacks, machine plus libre) ; le coût par frame, lui, échoue.
        self.assertTrue(self.compare(
            self.metrics(advance_ms_avg=1.2), self.metrics(advance_ms_avg=1.0)))
        self.assertEqual(self.compare(
            self.metrics(advance_ms_avg=1.05, load_ms_avg=0.4), self.metrics(advance_ms_avg=1.0, load_ms_avg=0.5)), [])

    def test_measurements_without_costs_are_compared_on_fps_only(self):
        self.assertEqual(self.compare(self.metrics(advance_ms_avg=3.0), self.metrics()), [])

    def test_rollback_ceiling_requires_the_measure(self):
        self.assertTrue(self.compare(self.metrics(), self.metrics(), max_rollback=40))
        self.assertTrue(self.compare(
            self.metrics(rollback_ms_worst=45.0), self.metrics(), max_rollback=40))
        self.assertEqual(self.compare(
            self.metrics(rollback_ms_worst=35.0), self.metrics(), max_rollback=40), [])


class P2PTraceTests(unittest.TestCase):
    def test_missing_empty_truncated_and_mismatched_traces_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            a, b = (Path(directory) / name for name in ("a.trace", "b.trace"))
            a.write_text("0 " + "a" * 32 + " 10\n1 " + "b" * 32 + " 11\n")
            with self.assertRaises(OSError):
                traces.check([a, b], 2)
            for content in ("", a.read_text().splitlines()[0] + "\n",
                            a.read_text().replace("b" * 32, "c" * 32)):
                b.write_text(content)
                with self.assertRaises(ValueError):
                    traces.check([a, b], 2)
            b.write_text(a.read_text())
            traces.check([a, b], 2)

    def test_duplicate_input_file_is_not_a_second_peer(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "a.trace"
            path.write_text("0 " + "a" * 32 + " 10\n")
            with self.assertRaises(ValueError):
                traces.check([path, path], 1)

    def test_non_contiguous_or_malformed_trace_fails(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "a.trace"
            for content in ("1 " + "a" * 32 + " 10\n", "0 invalid 10\n"):
                path.write_text(content)
                with self.assertRaises(ValueError):
                    traces.read_trace(path, 1)


class ShardPlanTests(unittest.TestCase):
    """`scenario-shard-plan.py` nomme les scénarios par leur id (`generated/<jeu>/<nom>`) :
    deux jeux peuvent générer le même nom court, et le runner refuse un nom ambigu."""

    def test_homonyms_of_two_games_are_both_planned_by_id(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative in ("idle", "generated/throne/weapon", "generated/gungeon/weapon"):
                path = root / "tests/scenarios" / f"{relative}.ron"
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("Scenario(frames: 60)\n")
            output = subprocess.run(
                [sys.executable, str(SCRIPTS / "scenario-shard-plan.py"), str(root),
                 str(root / "absent.json"), "2", ""],
                check=True, capture_output=True, text=True,
            ).stdout
            planned = {
                name for line in output.splitlines() for name in line.split("\t")[1].split(",") if name
            }
            self.assertEqual(
                planned, {"idle", "generated/throne/weapon", "generated/gungeon/weapon"}
            )



if __name__ == "__main__":
    unittest.main()
