import unittest
from rsp1_external import Case,translate,Unsupported,parse,eligible
import z3
SUM='''(set-logic LIA)(synth-fun f ((a Int)(b Int)) Int)
(declare-var x Int)(declare-var y Int)(constraint (= (f x y) (+ x y)))(check-synth)'''
I=lambda i:{'Input':i}
N=lambda n:{'Lit':{'Int':n}}
B=lambda op,a,b:{'Bin':[op,a,b]}
class Tests(unittest.TestCase):
 def test_correct_sum_proven(self):self.assertTrue(Case(SUM).check(B('Add',I(0),I(1)))['verified'])
 def test_wrong_sum_refuted(self):self.assertFalse(Case(SUM).check(B('Sub',I(0),I(1)))['verified'])
 def test_independent_teacher_inputs(self):
  t=Case(SUM).examples();self.assertEqual(len(t['examples']),16)
  for r in t['examples']:self.assertEqual(r['output'],sum(r['input']))
 def test_argument_binding(self):
  text=SUM.replace('(+ x y)','(- y x)');self.assertTrue(Case(text).check(B('Sub',I(1),I(0)))['verified']);self.assertFalse(Case(text).check(B('Sub',I(0),I(1)))['verified'])
 def test_zero_divisor_cannot_be_a_proof(self):self.assertFalse(Case(SUM).check(B('Rem',I(0),I(1)))['verified'])
 def test_overflow_rejected_even_if_mathematically_cancelled(self):
  big=B('Mul',B('Mul',I(0),I(0)),B('Mul',I(0),I(0)));out=B('Add',B('Sub',big,big),B('Add',I(0),I(1)))
  self.assertFalse(Case(SUM).check(out)['verified'])
 def test_conditional_lazy_safety(self):
  bad=B('Rem',I(0),N(0));expr={'If':[{'Lit':{'Bool':True}},B('Add',I(0),I(1)),bad]};self.assertTrue(Case(SUM).check(expr)['verified'])
 def test_signed_remainder(self):
  expr,s=translate(B('Rem',I(0),I(1)),[z3.IntVal(-7),z3.IntVal(3)]);self.assertEqual(z3.simplify(expr).as_long(),-1)
 def test_bitwise_semantics(self):
  expr,s=translate(B('Xor',I(0),I(1)),[z3.IntVal(-3),z3.IntVal(7)]);self.assertEqual(z3.simplify(expr).as_long(),-3^7)
 def test_extra_globals_are_explicitly_unsupported(self):
  with self.assertRaises(Unsupported):Case(SUM.replace('(check-synth)','(declare-var z Int)(check-synth)'))
 def test_nonuniform_calls_unsupported(self):
  with self.assertRaises(Unsupported):Case(SUM.replace('(check-synth)','(constraint (= (f y x) (+ x y)))(check-synth)'))
 def test_missing_constraint_not_vacuous_success(self):
  with self.assertRaises(Unsupported):Case('(synth-fun f ((x Int)) Int)(declare-var x Int)')
if __name__=='__main__':unittest.main()
