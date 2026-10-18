"""Protocol-only tests: no game rules, goals or solution data."""
import json, unittest
import numpy as np
from unified_arc3 import encode, dump
class RasterContract(unittest.TestCase):
    def test_numpy_and_list_are_identical(self):
        a=np.arange(64,dtype=np.uint8).reshape(8,8)%16
        self.assertEqual(encode(a),encode(a.tolist()))
    def test_exact_bounded_features(self):
        self.assertEqual(encode(np.zeros((64,64),dtype=np.uint8)),[4096]+[0]*31)
    def test_empty_rejected(self):
        with self.assertRaises(ValueError):encode(np.zeros((0,0),dtype=np.uint8))
    def test_ragged_rejected(self):
        with self.assertRaises(ValueError):encode([[0]*4,[0]*3,[0]*4,[0]*4])
    def test_invalid_palette_rejected(self):
        with self.assertRaises(ValueError):encode([[16]*4]*4)
    def test_small_image_rejected(self):
        with self.assertRaises(ValueError):encode([[0]*3]*3)
    def test_numpy_serializes_without_losing_pixels(self):
        x=np.arange(16,dtype=np.int64).reshape(4,4)
        self.assertEqual(json.loads(json.dumps(dump({'frame':x}))),{'frame':x.tolist()})
    def test_excluded_sdk_frame_is_explicitly_preserved(self):
        class FakeFrame:
            frame=np.zeros((1,4,4),dtype=np.uint8)
            def model_dump(self,mode):return {'state':'NOT_FINISHED'}
        self.assertEqual(dump(FakeFrame())['frame'],[[[0]*4]*4])
if __name__=='__main__':unittest.main()
