#!/usr/bin/env python3
"""Independent arithmetic audit of saved exam decisions, no model calls."""
import hashlib,json,sys,re
from pathlib import Path
p=Path(sys.argv[1]);summary=json.loads((p/'summary.json').read_text(encoding='utf-8'))
actual=hashlib.sha256((p/'learned-school.json').read_bytes()).hexdigest()
assert actual==summary['frozen_checkpoint_sha256']
counts={}
for line in (p/'authored-exam.jsonl').read_text(encoding='utf-8').splitlines():
 row=json.loads(line)
 for arm,answer in row['answers'].items():
  d=counts.setdefault(row['category'],{}).setdefault(arm,{'count':0,'correct':0,'answered':0})
  d['count']+=1;d['correct']+=answer['answer']==row['expected'];d['answered']+=answer['answer'] is not None
assert counts==summary['checks'],(counts,summary['checks'])
external={}
f=p/'external-exam.jsonl'
if f.exists():
 for line in f.read_text(encoding='utf-8').splitlines():
  row=json.loads(line);d=external.setdefault(row['config'],{'questions':0,'correct':0,'answered':0,'training_question_overlap':0,'clean_questions':0,'clean_correct':0,'uniform_guess_expected_correct':0.0})
  correct=row['selected']==row['expected'];assert correct==row['correct']
  d['questions']+=1;d['correct']+=correct;d['answered']+=row['selected'] is not None
  d['training_question_overlap']+=row['overlap'];d['clean_questions']+=not row['overlap'];d['clean_correct']+=correct and not row['overlap'];d['uniform_guess_expected_correct']+=1/len(row['choices']['text'])
 assert external==summary['external']
print(json.dumps({'checks':counts,'external':external,'checkpoint_hash':actual,'recount':'PASS'},indent=2))
