import importlib.util
from pathlib import Path
import unittest

PATH = Path(__file__).resolve().parents[1] / 'scripts' / 'external_cue_access_audit.py'
spec = importlib.util.spec_from_file_location('cue_access_audit', PATH)
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


def record(swapped=False, *, selected_upper=True, exposed=True):
    upper, lower = [5, 2], [5, 4]
    target, other = (lower, upper) if swapped else (upper, lower)
    selected = upper if selected_upper else lower
    return {'success_position': target, 'failure_position': other,
            'terminal_position': selected, 'success': selected == target,
            'cue_visible_steps': [0] if exposed else [],
            'input_sha256': ['cue-b' if swapped and exposed else 'cue-a', 'fork'],
            'actions': [2, 0 if selected_upper else 1]}


class ScoreContracts(unittest.TestCase):
    def test_constant_side_never_proves_memory(self):
        pairs = [{'original': record(), 'cue_swapped': record(True)} for _ in range(32)]
        s = audit.score_pairs(pairs)
        self.assertEqual(s['successes'], 32)
        self.assertEqual(s['both_correct'], 0)
        self.assertFalse(s['cue_use_supported'])

    def test_correct_paired_switches_can_pass(self):
        pairs = [{'original': record(), 'cue_swapped': record(True, selected_upper=False)}
                 for _ in range(32)]
        s = audit.score_pairs(pairs)
        self.assertTrue(s['cue_use_supported'])
        self.assertEqual(s['both_correct'], 32)
        self.assertEqual(s['exit_switch_pairs'], 32)

    def test_absent_cue_is_not_mislabeled_as_forgotten(self):
        pair = {'original': record(exposed=False), 'cue_swapped': record(True, exposed=False)}
        s = audit.score_pairs([pair])
        self.assertEqual(s['unexposed_pairs'], 1)
        self.assertEqual(s['identical_input_pairs'], 1)
        self.assertEqual(s['exposed_pairs'], 0)

    def test_identical_history_with_different_action_is_invalid(self):
        pair = {'original': record(exposed=False),
                'cue_swapped': record(True, selected_upper=False, exposed=False)}
        with self.assertRaisesRegex(ValueError, 'Identical input histories'):
            audit.score_pairs([pair])

    def test_side_labels_must_exchange(self):
        with self.assertRaisesRegex(ValueError, 'did not exchange'):
            audit.score_pairs([{'original': record(), 'cue_swapped': record()}])

    def test_false_unobserved_claim_is_detected(self):
        a = record(exposed=False)
        b = record(True, exposed=False)
        b['input_sha256'] = ['DIFFERENT']
        with self.assertRaisesRegex(ValueError, 'Unseen cue affected'):
            audit.score_pairs([{'original': a, 'cue_swapped': b}])


if __name__ == '__main__':
    unittest.main()
