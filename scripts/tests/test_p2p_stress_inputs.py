"""Bornes exactes des `--hold` de scripts/p2p-stress-inputs.py."""
import importlib.util
from pathlib import Path
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "p2p-stress-inputs.py"
spec = importlib.util.spec_from_file_location("p2p_stress_inputs", SCRIPT)
inputs = importlib.util.module_from_spec(spec)
spec.loader.exec_module(inputs)


class HoldBoundsTests(unittest.TestCase):
    def test_hold_covers_exactly_its_range(self):
        inputs._test_bounds()


if __name__ == "__main__":
    unittest.main()
