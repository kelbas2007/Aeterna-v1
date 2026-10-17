#!/usr/bin/env python3
"""Grounded composition, paired mechanism ablation, and fail-closed ambiguity.

All text/program reference equations below belong to the teacher/evaluator.
Rust receives ONLY scalar Q/A lessons. Entire composite questions are withheld
until freeze. This is authored developmental evidence, not a human-age exam.
"""
from __future__ import annotations
import argparse, hashlib, json, random, sys
from pathlib import Path
from education_experiment import Client, TOPICS, pair, one, quotient, write_json, sha

PROTOCOL = {
    'name': 'SG1-grounded-composition', 'independent_external_benchmark': False,
    'teacher_seed': 718312, 'exam_seed': 810917,
    'composite_training_examples': 0, 'exam_feedback_to_learner': False,
    'prior_wording_exam_is_now_remedial_training': True,
    'comparison': 'same acquired program library and same expanded lessons; disable compositional grounding only',
    'unanswered_solvable_counts_as_wrong': True,
    'claims_human_understanding': False, 'scope': 'learned integer language, executable composition, interpretation monitoring',
}
# Isolated phrases, not test strings or test expression trees.
FRAGMENTS = [
    ('сумма {0} и {1}',lambda a,b:a+b,pair),
    ('суммы {0} и {1}',lambda a,b:a+b,pair),
    ('произведение {0} и {1}',lambda a,b:a*b,pair),
    ('произведения {0} и {1}',lambda a,b:a*b,pair),
    ('разность {0} и {1}',lambda a,b:a-b,pair),
    ('разности {0} и {1}',lambda a,b:a-b,pair),
    ('частное {0} и {1}',lambda a,b:a//b,quotient),
    ('частного {0} и {1}',lambda a,b:a//b,quotient),
    ('sum of {0} and {1}',lambda a,b:a+b,pair),
    ('product of {0} and {1}',lambda a,b:a*b,pair),
    ('difference between {0} and {1}',lambda a,b:a-b,pair),
    ('quotient of {0} and {1}',lambda a,b:a//b,quotient),
    ('вычти {0} из {1}',lambda a,b:b-a,pair),
    ('вычти из {0} {1}',lambda a,b:a-b,pair),
    ('subtract {0} from {1}',lambda a,b:b-a,pair),
    ('{0} минус {1}',lambda a,b:a-b,pair),
    ('вычисли {0}',lambda a:a,one),
    ('каков результат {0}',lambda a:a,one),
    ('what is {0}',lambda a:a,one),
    ('compute {0}',lambda a:a,one),
    ('площади прямоугольника со сторонами {0} и {1}',lambda a,b:a*b,pair),
    ('стоимость {0} предметов по {1}',lambda a,b:a*b,pair),
    ('fep {0} kor {1}',lambda a,b:a+b,pair),
    ('lut {0} vek {1}',lambda a,b:a*b,pair),
]

def unique_examples(rng,form,func,sampler,count,source):
    seen=set();rows=[]
    while len(rows)<count:
        args=sampler(rng,3)
        if args in seen:continue
        seen.add(args);rows.append(dict(question=form.format(*args),answer=func(*args),source=source))
    return rows

def corpus(prior:Path|None):
    lessons=[];facts=[]
    if prior:
        saved=json.loads(prior.read_text(encoding='utf-8'));school=saved['cognition']['education']
        for s in school['skills']:lessons.append(s['training']+s['validation'])
        facts=school['facts']
    else:
        rng=random.Random(73419)
        for topic in TOPICS:
            for form in topic.forms:
                # Development fallback does not include Ai2 factual memory.
                lessons.append(unique_examples(rng,form,topic.answer,topic.sample,64,'authored-school:'+topic.name))
        facts=[dict(question=e['question'],answer=str(e['answer']),source=e['source']) for x in lessons for e in x]
    rng=random.Random(PROTOCOL['teacher_seed'])
    for topic in TOPICS:
        lessons.append(unique_examples(rng,topic.novel,topic.answer,topic.sample,32,'remedial-language:'+topic.name))
    for form,f,sampler in FRAGMENTS:
        lessons.append(unique_examples(rng,form,f,sampler,32,'isolated-language-lesson:'+form))
    return lessons,facts

# Evaluation-only recursive rendering. No generated string below is sent to teach.
OPS = [
 ('сумма {0} и {1}',lambda a,b:a+b),
 ('произведение {0} и {1}',lambda a,b:a*b),
 ('разность {0} и {1}',lambda a,b:a-b),
 ('sum of {0} and {1}',lambda a,b:a+b),
 ('product of {0} and {1}',lambda a,b:a*b),
 ('difference between {0} and {1}',lambda a,b:a-b),
 ('вычти {0} из {1}',lambda a,b:b-a),
 ('subtract {0} from {1}',lambda a,b:b-a),
 ('fep {0} kor {1}',lambda a,b:a+b),
 ('lut {0} vek {1}',lambda a,b:a*b),
]
def composite(rng,depth):
    value=rng.randint(2,29);text=str(value);signature=[]
    for _ in range(depth):
        op=rng.randrange(len(OPS));v=rng.randint(2,29);flip=bool(rng.randrange(2))
        if flip:a,b= value,v;sa,sb='('+text+')',str(v)
        else:a,b=v,value;sa,sb=str(v),'('+text+')'
        text=OPS[op][0].format(sa,sb);value=OPS[op][1](a,b);signature.append((op,flip))
    wrapper=rng.choice(['вычисли {0}','каков результат {0}','what is {0}','compute {0}'])
    return wrapper.format(text),value,signature

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);ap.add_argument('--prior',type=Path);args=ap.parse_args()
    root=args.output;root.mkdir(parents=True,exist_ok=False)
    write_json(root/'protocol.json',PROTOCOL)
    lessons,facts=corpus(args.prior)
    write_json(root/'teacher.json',lessons)
    provenance={'prior_checkpoint_sha256':sha(args.prior.read_bytes()) if args.prior else None,'facts':len(facts)}
    write_json(root/'prior.json',provenance)
    client=Client(args.binary,root/'teacher-process.log');client.send('init');client.send('enable_composition');fits=[]
    try:
        for i,examples in enumerate(lessons):
            fit=client.send('teach',examples=examples)['result'];fits.append(fit)
            if not fit['accepted']:raise RuntimeError('Lesson failed before exam: '+fit['pattern'])
            if i%16==0:print('validated lesson',i,file=sys.stderr,flush=True)
        for start in range(0,len(facts),256):client.send('facts',facts=facts[start:start+256])
        write_json(root/'lessons.json',fits);client.send('freeze');checkpoint=client.send('checkpoint')['checkpoint']
    finally:client.close()
    (root/'learned-semantic.json').write_bytes(checkpoint.encode())
    frozen=sha(checkpoint.encode());write_json(root/'frozen.json',{'sha256':frozen,'exam_not_started':True})
    client=Client(args.binary,root/'exam-process.log');client.send('load',checkpoint=checkpoint)
    cold=Client(args.binary,root/'cold-process.log');cold.send('init');cold.send('enable_composition');cold.send('freeze')
    counts={};seen={e['question'].lower() for xs in lessons for e in xs};unique=set();rows=[]
    def assess(category,q,expected,metadata=None):
        if q.lower() in seen:raise RuntimeError('Training/exam collision')
        if q.lower() in unique:raise RuntimeError('Repeated exam question')
        unique.add(q.lower())
        full=client.send('ask',question=q)['result'];surface=client.send('ask_surface',question=q)['result'];initial=cold.send('ask',question=q)['result']
        expected=None if expected is None else str(expected)
        row={'category':category,'question':q,'expected':expected,'full':full,'same_knowledge_surface':surface,'cold':initial}
        if metadata:row.update(metadata)
        rows.append(row)
        with (root/'exam.jsonl').open('a',encoding='utf-8') as journal:journal.write(json.dumps(row,ensure_ascii=False)+'\n')
        for name,result in [('full',full),('same_knowledge_surface',surface),('cold',initial)]:
            m=counts.setdefault(category,{}).setdefault(name,dict(total=0,correct=0,answered=0,wrong_answered=0))
            m['total']+=1;m['correct']+=result['answer']==expected;m['answered']+=result['answer'] is not None
            m['wrong_answered']+=result['answer'] is not None and result['answer']!=expected
    try:
        rng=random.Random(PROTOCOL['exam_seed'])
        for depth in [2,3,4,5]:
            for _ in range(128):
                q,expected,signature=composite(rng,depth)
                assess('unseen_composition_depth_'+str(depth),q,expected,{'structure':signature})
        # Established original wording regressions, kept separate from composition.
        for topic in TOPICS:
            emitted=0
            while emitted<8:
                a=topic.sample(rng,1101);q=topic.novel.format(*a)
                if q.lower() in unique or q.lower() in seen:continue
                assess('remedial_wording_regression',q,topic.answer(*a));emitted+=1
        # Actual same known words and noncommutative argument roles, composed inputs.
        for _ in range(32):
            a,b,c=rng.sample(range(101,999),3)
            assess('role_binding',f'Вычти разность {a} и {b} из {c}',c-(a-b))
            assess('role_binding',f'Вычти из разности {a} и {b} {c}',a-b-c)
        for k in range(32):
            a=103+k;b=31+k
            assess('ambiguity_refusal',f'{a} минус {b} минус 0',None)
            assess('explicit_disambiguation',f'({a} минус {b}) минус 2',a-b-2)
            assess('explicit_disambiguation',f'{a} минус ({b} минус 2)',a-b+2)
            assess('missing_argument_refusal',f'сумма {a} и',None)
            assess('unlearned_negation_refusal',f'не сумма {a} и {b}',None)
            assess('extra_number_refusal',f'сумма {a} и {b} 999',None)
        demo=[]
        for q in [
          'Сложи числа 17 и 23.',
          'Вычти из суммы 17 и 23 произведение 3 и 4.',
          'Вычти из 100 стоимость 3 предметов по 7.',
          'Площадь прямоугольника со сторонами (сумма 2 и 5) и (разность 11 и 8).',
          '4 минус 2 минус 0',
          '12 минус (6 минус 2)',
          'Не сложи числа 17 и 23.',
          'Почему зимой холоднее, чем летом?',
        ]:
            demo.append(dict(question=q,answer=client.send('ask',question=q)['result'],understanding=client.send('understand',question=q)['result']))
        clarification=[]
        for i in range(32):
            q=f'{301+i} минус {17+i} минус 2'
            u=client.send('understand',question=q)['result']
            probe=u['distinguishing_probe']
            if not probe:raise RuntimeError('Ambiguity did not produce a question')
            x,y,z=probe['inputs'];reply=(x-y)-z
            # Teacher answers the requested COUNTEREXAMPLE, not the original exam.
            result=client.send('clarify',question=q,reply=reply)['result']
            clarification.append(dict(question=q,probe=probe,teacher_reply=reply,
                expected=str((301+i)-(17+i)-2),result=result))
        write_json(root/'clarification.json',clarification)
        write_json(root/'examples.json',demo)
        if client.send('checkpoint')['checkpoint']!=checkpoint:raise RuntimeError('Frozen exam mutated cognition')
        write_json(root/'summary.json',{'protocol':PROTOCOL,'lessons':len(fits),'new_teaching_examples':sum(len(x) for x in lessons[52:]),'teacher_examples':sum(len(x) for x in lessons),'facts_preserved':len(facts),'counts':counts,'checkpoint_sha256':frozen,'checkpoint_unchanged':True,'school':client.send('school_report')['report']})
        with (root/'exam.jsonl').open('w',encoding='utf-8') as f:
            for row in rows:f.write(json.dumps(row,ensure_ascii=False)+'\n')
        print(json.dumps(counts,ensure_ascii=False,indent=2))
    finally:client.close();cold.close()
if __name__=='__main__':main()
