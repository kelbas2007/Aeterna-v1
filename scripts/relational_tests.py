#!/usr/bin/env python3
"""Mechanism tests, not benchmark evidence of acquired intelligence."""
import unittest,collections,json,tempfile
from pathlib import Path
import numpy as np
from relational_train import norm,read_prefix,numeric,relation,OPS,ff,chart,export_factor
from relational_external import scalar_answer,asdiv_complement,text_key
from word_problem_train import ground_raw
class Tests(unittest.TestCase):
    def test_prefix_is_data_not_python(self):
        with self.assertRaises(ValueError):read_prefix("__import__('os')")
    def test_grouped_subtraction_normalizes_correctly(self):
        e=norm(('-',0,('-',1,2)))
        self.assertEqual(numeric(e,[20,7,3]),16)
        self.assertEqual(relation(e,0,1),'-');self.assertEqual(relation(e,0,2),'+')
    def test_division_normalizes_without_float_fitting(self):
        e=norm(('/',0,('/',1,2)))
        self.assertEqual(numeric(e,[20,10,3]),6)
    def test_quantity_replacement_invariant(self):
        a,_=ground_raw('Have 7 red and 13 blue. How many red?');b,_=ground_raw('Have 1 red and 999 blue. How many red?')
        self.assertEqual(ff(a,2,0,1),ff(b,2,0,1))
    def test_query_not_a_bag(self):
        a,_=ground_raw('Have 7 red and 13 blue. How many red?');b,_=ground_raw('Have 7 red and 13 blue. How many blue?')
        self.assertNotEqual(ff(a,2,0,1),ff(b,2,0,1))
    def test_scalar_gold_does_not_select_first_of_many_answers(self):
        self.assertIsNone(scalar_answer('3, 4 (boys, girls)'));self.assertIsNone(scalar_answer('2 1/2 (cups)'))
        self.assertEqual(scalar_answer('2.5 (cups)'),'5/2');self.assertEqual(scalar_answer('1,200 (apples)'),'1200')
    def test_overlap_membership_does_not_use_labels(self):
        with tempfile.TemporaryDirectory() as t:
            p=Path(t)/'x.xml';p.write_text('<ASDiv><Problem ID="a"><Body>Have 7 red.</Body><Question>How many?</Question><Answer>7</Answer></Problem><Problem ID="b"><Body>Have 13 blue.</Body><Question>How many?</Question><Answer>13</Answer></Problem></ASDiv>')
            a,m=asdiv_complement(p,[{'question':'Have 999 red. How many?'}]);self.assertEqual(len(a),1);self.assertEqual(a[0]['source_id'],'b');self.assertEqual(m['excluded'][0]['reason'],'masked_training_overlap')
    def test_tree_search_combines_four_inputs_not_a_bank(self):
        r={'values':[2,3,5,7]};u=np.array([[-20.,20.]]*4);s=np.full((6,7),-10.);s[:,OPS.index('+')]=10
        best=chart(r,[u,s])[0];self.assertEqual(best[2],17)
    def test_mixed_graph_recovers_its_consistent_expression(self):
        e=norm(('-',('*',0,1),('+',2,3)));r={'values':[11,7,2,3]};u=np.array([[-50.,50.]]*4);s=np.full((6,7),-20.)
        from itertools import combinations
        for k,(a,b) in enumerate(combinations(range(4),2)):s[k,OPS.index(relation(e,a,b))]=20
        best=chart(r,[u,s])[0];self.assertEqual(best[2],72)
    def test_irrelevant_factor_not_forced_into_expression(self):
        r={'values':[2,99,5]};u=np.array([[-50.,50.],[50.,-50.],[-50.,50.]])
        s=np.full((3,7),-10.);s[0,6]=10;s[1,0]=10;s[2,6]=10
        best=chart(r,[u,s])[0];self.assertEqual(best[2],7)
if __name__=='__main__':unittest.main()
