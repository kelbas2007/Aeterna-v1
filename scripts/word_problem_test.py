import unittest,tempfile,csv
from pathlib import Path
from fractions import Fraction as F
from word_problem_train import ground_raw,load_training,candidates,induce
from word_problem_verify import equal_program,run,gold_tree,quantities
class Tests(unittest.TestCase):
    def test_raw_unmasked_quantity(self):self.assertEqual(ground_raw('A 300 home lost 8 % in a 1-year period')[1],['300','8','1'])
    def test_brand_digits(self):self.assertEqual(ground_raw('MP3 costs 40.0 plus 2.4 tax')[1],['40.0','2.4'])
    def test_teacher_equation_not_consumed(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'train.csv'
            with p.open('w') as f:
                w=csv.DictWriter(f,fieldnames=['Question','Numbers','Answer','Equation','group_nums']);w.writeheader();w.writerow({'Question':'has number0 and number1','Numbers':'2 3','Answer':'5','Equation':'DONT_READ_OR_EXECUTE','group_nums':'INVALID'})
            rows=load_training(p);self.assertTrue(induce(rows)[0]);self.assertNotIn('Equation',rows[0])
    def test_arithmetic_new_values(self):self.assertEqual(run({'Bin':['Sub',{'Input':0},{'Input':1}]},[F(101),F(32)]),69)
    def test_equivalent_compositions(self):
        a={'Bin':['Sub',{'Input':0},{'Bin':['Sub',{'Input':1},{'Input':2}]}]}
        b={'Bin':['Add',{'Bin':['Sub',{'Input':0},{'Input':1}]},{'Input':2}]}
        self.assertTrue(equal_program(a,b,3))
    def test_accidental_numeric_match_not_equivalent(self):self.assertFalse(equal_program({'Bin':['Add',{'Input':0},{'Input':1}]},{'Bin':['Mul',{'Input':0},{'Input':1}]},2))
    def test_ambiguous_gold_binding(self):
        with self.assertRaises(ValueError):gold_tree('(3 + 3)',[F(3),F(3)])
    def test_independent_quantity_extraction(self):self.assertEqual(quantities('MP3 price $1,203.50 and 5'),[F('1203.5'),F(5)])
    def test_search_bound(self):self.assertTrue(all(len(p)<=5 for p in candidates(4)))
if __name__=='__main__':unittest.main()
