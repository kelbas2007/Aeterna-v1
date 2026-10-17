#!/usr/bin/env python3
"""Post-exam numeric-substitution diagnostic; never updates learned weights."""
import argparse,json,re
from pathlib import Path
from word_problem_external import Client,dump
PATTERN=re.compile(r'(?<![A-Za-z0-9])[-+]?(?:\d{1,3}(?:,\d{3})+|\d+)(?:\.\d+)?')
VALUES=['101','137','173','211','251','293']
def main():
 p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--evidence',type=Path,required=True);a=p.parse_args();root=a.evidence
 c=Client(a.binary.resolve(),root/'invariance.native.log');state=(root/'frozen/learned-relations.json').read_text();c.request({'op':'load','checkpoint':state});report={}
 for name in ['SVAMP-regression','ASDiv-complement']:
  rows=[json.loads(s) for s in (root/'external'/name/'relations.jsonl').read_text().splitlines()];out=[]
  for r in rows:
   text=r['question'];n=len(PATTERN.findall(text));old=r['prediction']
   if not 1<=n<=6 or not old['interpretations']:continue
   it=iter(VALUES);newtext=PATTERN.sub(lambda _:next(it),text)
   new=c.request({'op':'ask_relational','question':newtext})['result']
   scores=(old['unary_scores']==new['unary_scores'] and [x['scores'] for x in old['relations']]==[x['scores'] for x in new['relations']])
   same=bool(new['interpretations']) and old['interpretations'][0]['program']==new['interpretations'][0]['program']
   out.append({'index':r['index'],'same_factor_scores':scores,'same_selected_expression':same})
  report[name]={'eligible_original_answers':len(out),'identical_factor_scores':sum(x['same_factor_scores'] for x in out),'identical_selected_expression':sum(x['same_selected_expression'] for x in out),'cases':out}
 assert c.request({'op':'checkpoint'})['checkpoint']==state;c.close()
 report['not_independent_accuracy']=True;report['caveat']='This tests number-value invariance, not linguistic understanding. Substitution may change zero-divisor validity; no synthetic answers count as external successes.'
 dump(root/'numeric-invariance.json',report)
 print(json.dumps({k:{z:v for z,v in d.items() if z!='cases'} for k,d in report.items() if isinstance(d,dict)},indent=2))
if __name__=='__main__':main()
