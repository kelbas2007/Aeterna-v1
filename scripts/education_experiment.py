#!/usr/bin/env python3
"""Reproducible teaching, frozen controls and external school-QA diagnostic.

The Rust learner receives examples, never these reference equations. Written
phrasing is a controlled curriculum, not a representative human-age assessment.
Test labels are held by this evaluator only. No evaluation outcome is sent as a
teaching command. Training and test evidence are saved separately.
"""
from __future__ import annotations
import argparse, hashlib, json, random, re, subprocess, sys, time
from pathlib import Path
from dataclasses import dataclass
from typing import Callable

@dataclass
class Topic:
    name: str
    forms: tuple[str, str]
    sample: Callable[[random.Random, int], tuple[int, ...]]
    answer: Callable[..., int]
    novel: str

def pair(r, b): return (r.randint(b,b+59),r.randint(b,b+59))
def triple(r,b): return (*pair(r,b),r.randint(b,b+29))
def quotient(r,b):
    d=r.randint(b,b+29); return (d*r.randint(b,b+39),d)
def payment(r,b):
    price,quantity=pair(r,b);return (price*quantity+r.randint(b,b+49),price,quantity)
def triangle(r,b): return (2*r.randint(b,b+29),r.randint(b,b+29))
def percent(r,b): return (100*r.randint(b,b+49),r.randint(1,99))
def mean(r,b):
    a,c,d=triple(r,b); return (a,c,d+(-(a+c+d))%3)
def fraction(r,b):
    a=r.randint(1,39);d=r.randint(b,b+49);return (a,d,d*r.randint(b,b+29))
def one(r,b): return (r.randint(b,b+199),)
TOPICS = [
 Topic('addition',('Найди сумму {} и {}.','Find the sum of {} and {}.'),pair,lambda a,b:a+b,'Сложи числа {} и {}.'),
 Topic('subtraction',('Вычти {} минус {}.','Find {} minus {}.'),pair,lambda a,b:a-b,'Насколько {} больше {}?'),
 Topic('multiplication',('Найди произведение {} и {}.','Find the product of {} and {}.'),pair,lambda a,b:a*b,'Умножь {} на {}.'),
 Topic('division',('Раздели {} на {}.','Divide {} by {}.'),quotient,lambda a,b:a//b,'Сколько групп по {1} помещается в {0}?'),
 Topic('remainder',('Найди остаток от деления {} на {}.','Find the remainder of {} divided by {}.'),pair,lambda a,b:a%b,'Раздай {} предметов по {}. Сколько останется?'),
 Topic('rectangle_area',('Площадь прямоугольника со сторонами {} и {}.','Area of a rectangle with sides {} and {}.'),pair,lambda a,b:a*b,'Прямоугольная комната имеет длину {} и ширину {}. Какова площадь?'),
 Topic('rectangle_perimeter',('Периметр прямоугольника со сторонами {} и {}.','Perimeter of a rectangle with sides {} and {}.'),pair,lambda a,b:2*(a+b),'Сколько ограды нужно вокруг прямоугольника длиной {} и шириной {}?'),
 Topic('cuboid_volume',('Объём коробки с размерами {} {} {}.','Volume of a box with dimensions {} {} {}.'),triple,lambda a,b,c:a*b*c,'Найди вместимость ящика длиной {} шириной {} высотой {}.'),
 Topic('square_area',('Площадь квадрата со стороной {}.','Area of a square with side {}.'),one,lambda a:a*a,'У квадратного участка сторона {}. Чему равна площадь?'),
 Topic('cube_volume',('Объём куба с ребром {}.','Volume of a cube with edge {}.'),one,lambda a:a*a*a,'Сколько кубических единиц в кубе с ребром {}?'),
 Topic('triangle_area',('Площадь треугольника с основанием {} и высотой {}.','Area of a triangle with base {} and height {}.'),triangle,lambda a,b:a*b//2,'Основание треугольника равно {} а высота {}. Вычисли площадь.'),
 Topic('percentage',('Найди от {} величину {} процентов.','Find of {} the amount {} percent.'),percent,lambda a,b:a*b//100,'Чему равны {1} процентов от {0}?'),
 Topic('unit_price',('Цена одной вещи {} количество {}. Найди стоимость.','Unit price {} quantity {}. Find total cost.'),pair,lambda a,b:a*b,'Нужно купить {} товаров по {} каждый. Сколько заплатить?'),
 Topic('change',('Оплачено {} цена вещи {} количество {}. Найди сдачу.','Paid {} unit price {} quantity {}. Find change.'),payment,lambda a,b,c:a-b*c,'Из суммы {} вычти стоимость {} предметов по {}.'),
 Topic('mean',('Среднее арифметическое чисел {} {} {}.','Arithmetic mean of {} {} {}.'),mean,lambda a,b,c:(a+b+c)//3,'Найди среднее значение набора: {} {} {}.'),
 Topic('hours_minutes',('Переведи {} часов в минуты.','Convert {} hours to minutes.'),one,lambda a:a*60,'Сколько минут составляет {} часов?'),
 Topic('minutes_seconds',('Переведи {} минут в секунды.','Convert {} minutes to seconds.'),one,lambda a:a*60,'Вырази {} минут в секундах.'),
 Topic('metres_centimetres',('Переведи {} метров в сантиметры.','Convert {} metres to centimetres.'),one,lambda a:a*100,'Сколько сантиметров в {} метрах?'),
 Topic('kilograms_grams',('Переведи {} килограммов в граммы.','Convert {} kilograms to grams.'),one,lambda a:a*1000,'Вырази {} килограммов в граммах.'),
 Topic('sequence',('Начальное значение {} шаг {} число шагов {}. Найди итог.','Start {} increment {} number of steps {}. Find final value.'),triple,lambda a,b,c:a+b*c,'Начни с {} затем {} раз прибавь {}.'),
 Topic('speed',('Расстояние {} время {}. Найди скорость.','Distance {} time {}. Find speed.'),quotient,lambda a,b:a//b,'За время {1} пройдено расстояние {0}. Какова скорость?'),
 Topic('distance',('Скорость {} время {}. Найди расстояние.','Speed {} time {}. Find distance.'),pair,lambda a,b:a*b,'Какое расстояние пройдено за {} при скорости {}?'),
 Topic('density',('Масса {} объём {}. Найди плотность.','Mass {} volume {}. Find density.'),quotient,lambda a,b:a//b,'Объём тела {1} а масса {0}. Чему равна плотность?'),
 Topic('fraction_of',('Найди {} / {} от {}.','Find {} / {} of {}.'),fraction,lambda a,b,c:a*c//b,'Найди долю {0} из {1} от количества {2}.'),
 Topic('discount',('Цена {} скидка {} процентов. Найди новую цену.','Price {} discount {} percent. Find new price.'),percent,lambda a,b:a-a*b//100,'Какова цена после скидки {1} процентов с суммы {0}?'),
 Topic('shipping',('Цена вещи {} количество {} доставка {}. Найди общую стоимость.','Unit price {} quantity {} shipping {}. Find total cost.'),triple,lambda a,b,c:a*b+c,'Сколько стоят {} вещей по {} с доставкой {}?'),
]
# Every family and control is declared before any external test is loaded.
PROTOCOL={
 'kind':'education-development-not-human-age-norm', 'seed':73419,
 'teacher_examples_per_wording':64, 'teacher_fit_fraction':0.75,
 'new_number_questions_per_wording':64,'unseen_wording_per_topic':8,
 'topics':[t.name for t in TOPICS],
 'control':'same checkpoint, disable induced computation, retain teacher records',
 'external':'all ARC-Easy and ARC-Challenge TEST questions, after training on TRAIN only',
 'unanswered_counts_as_incorrect':True,'test_feedback_to_learner':False,
 'claim_12_year_old_level':False,'llm_calls_by_learner':0,
 'arithmetic_scope':'integer inputs and exact integer answers; rational/decimal arithmetic not supported',
}
class Client:
    def __init__(self,binary:Path,err:Path):
        self.err=err.open('w',encoding='utf-8')
        self.p=subprocess.Popen([str(binary.resolve())],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=self.err,text=True,encoding='utf-8',bufsize=1)
    def send(self,op:str,**kwargs):
        if self.p.poll() is not None:raise RuntimeError(f'Rust process exited {self.p.returncode}')
        self.p.stdin.write(json.dumps({'op':op,**kwargs},ensure_ascii=False)+'\n');self.p.stdin.flush()
        line=self.p.stdout.readline()
        if not line:raise RuntimeError('Rust transport closed')
        response=json.loads(line)
        if response.get('type')=='error':raise RuntimeError(response['message'])
        return response
    def close(self):
        try:
            if self.p.poll() is None:self.send('quit')
        finally:
            try:self.p.wait(timeout=10)
            except subprocess.TimeoutExpired:self.p.kill();self.p.wait()
            self.err.close()

def write_json(path,value):path.write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha(data:bytes):return hashlib.sha256(data).hexdigest()
def normal(s):return ' '.join(re.sub(r'[^\w+*/%=<>-]+',' ',s.lower(),flags=re.UNICODE).split())
def fetch(url):
    from urllib.request import Request,urlopen
    for attempt in range(3):
        try:
            with urlopen(Request(url,headers={'User-Agent':'AETERNA-education-research/1.0'}),timeout=45) as r:return r.read()
        except Exception:
            if attempt==2:raise
            time.sleep(1+attempt)

def external_source(root):
    """Resolve the upstream revision once. TEST files remain unread until freeze."""
    metadata=json.loads(fetch('https://huggingface.co/api/datasets/allenai/ai2_arc'))
    revision=metadata['sha'];files=[x['rfilename'] for x in metadata['siblings']]
    selected={}
    for config in ['ARC-Easy','ARC-Challenge']:
        for split in ['train','test']:
            paths=[p for p in files if p.startswith(config+'/') and p.split('/')[-1].startswith(split+'-') and p.endswith('.parquet')]
            if not paths:raise RuntimeError(f'Official dataset has no {config}/{split}')
            selected[config+'/'+split]=sorted(paths)
    source={'dataset':'allenai/ai2_arc','revision':revision,'paths':selected,'files':{}}
    write_json(root/'external-source.json',source);return source

def load_split(source,config,split,root):
    import pyarrow.parquet as pq
    import io
    result=[]
    for path in source['paths'][config+'/'+split]:
        url=f"https://huggingface.co/datasets/allenai/ai2_arc/resolve/{source['revision']}/{path}"
        data=fetch(url);source['files'][path]={'url':url,'sha256':sha(data),'bytes':len(data)}
        result.extend(pq.read_table(io.BytesIO(data)).to_pylist())
    write_json(root/'external-source.json',source);return result

def labelled_fact(row,source):
    choices=row['choices'];key=str(row['answerKey']);labels=list(map(str,choices['label']))
    if key not in labels:raise RuntimeError('Unmapped official answerKey')
    return {'question':row['question'],'answer':choices['text'][labels.index(key)],'source':source+':'+str(row['id'])}

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--output',type=Path,required=True)
    ap.add_argument('--external',action='store_true');args=ap.parse_args();root=args.output;root.mkdir(parents=True,exist_ok=False)
    (root/'DATA_NOTICE.txt').write_text("AI2 ARC question/answer material: Clark et al. (2018), Ai2, https://huggingface.co/datasets/allenai/ai2_arc . License CC-BY-SA-4.0, https://creativecommons.org/licenses/by-sa/4.0/ . Data are transformed into attributed question-answer memory; learned-school.json and external-exam.jsonl include this material. No endorsement implied. Code retains the repository license. Authored numerical teaching examples are original.\n",encoding='utf-8')
    write_json(root/'protocol.json',PROTOCOL)
    rng=random.Random(PROTOCOL['seed']);lessons=[];all_teacher=[];training_inputs=set()
    for topic in TOPICS:
        for form in topic.forms:
            examples=[];unique=set()
            while len(examples)<64:
                inputs=topic.sample(rng,2)
                if inputs in unique:continue
                unique.add(inputs);q=form.format(*inputs);training_inputs.add(q)
                examples.append({'question':q,'answer':topic.answer(*inputs),'source':'authored-school:'+topic.name})
            lessons.append((topic,form,examples));all_teacher.extend(examples)
    write_json(root/'authored-teacher.json',all_teacher)
    client=Client(args.binary,root/'teacher-process.log');client.send('init');fits=[]
    try:
        for topic,form,examples in lessons:
            result=client.send('teach',examples=examples)['result'];fits.append({'topic':topic.name,'wording':form,**result})
            print(f"lesson {topic.name}: {result['accepted']} validation {result['validation_correct']}/{result['validation']}",file=sys.stderr,flush=True)
        # The retrieval-only control receives the SAME examples, not an empty memory.
        facts=[{'question':r['question'],'answer':str(r['answer']),'source':r['source']} for r in all_teacher]
        source=None;external_train=[]
        if args.external:
            source=external_source(root)
            for config in ['ARC-Easy','ARC-Challenge']:
                rows=load_split(source,config,'train',root)
                external_train.extend(labelled_fact(r,'ai2-arc-train:'+config) for r in rows)
            facts.extend(external_train)
        for i in range(0,len(facts),256):client.send('facts',facts=facts[i:i+256])
        write_json(root/'lesson-results.json',fits);write_json(root/'before-freeze.json',client.send('school_report'))
        client.send('freeze');checkpoint=client.send('checkpoint')['checkpoint']
        (root/'learned-school.json').write_bytes(checkpoint.encode())
        frozen_hash=sha(checkpoint.encode());write_json(root/'frozen.json',{'sha256':frozen_hash,'external_test_not_loaded':True,'exam_started':False})
    finally:client.close()
    # Re-open a genuinely separate Rust process with the acquired knowledge.
    client=Client(args.binary,root/'exam-process.log');client.send('load',checkpoint=checkpoint)
    control=Client(args.binary,root/'control-process.log');control.send('load',checkpoint=checkpoint)
    cold=Client(args.binary,root/'cold-process.log');cold.send('init');cold.send('freeze')
    totals={};records=[];exam_rng=random.Random(5983401)
    def assess(category,q,expected):
        if q in training_inputs:raise RuntimeError('Teacher/exam overlap in generated task')
        outputs={'full':client.send('ask',question=q)['result'],'recall_only':control.send('ask',question=q,mode='RecallOnly')['result'],'cold':cold.send('ask',question=q)['result']}
        row={'category':category,'question':q,'expected':str(expected),'answers':outputs};records.append(row)
        for arm,out in outputs.items():
            m=totals.setdefault(category,{}).setdefault(arm,{'count':0,'correct':0,'answered':0})
            m['count']+=1;m['correct']+=out['answer']==str(expected);m['answered']+=out['answer'] is not None
    try:
        for topic,form,_ in lessons:
            unique=set()
            while len(unique)<64:
                values=topic.sample(exam_rng,1001)
                if values in unique:continue
                unique.add(values);assess('new_numbers',form.format(*values),topic.answer(*values))
        for topic in TOPICS:
            for _ in range(8):
                values=topic.sample(exam_rng,1001);assess('unseen_wording',topic.novel.format(*values),topic.answer(*values))
        with (root/'authored-exam.jsonl').open('w',encoding='utf-8') as f:
            for row in records:f.write(json.dumps(row,ensure_ascii=False)+'\n')
        recall=[]
        for f in facts[::max(1,len(facts)//100)][:100]:
            answer=client.send('ask',question=f['question'])['result'];recall.append({'question':f['question'],'correct':answer['answer']==f['answer'],'method':answer['method']})
        external={}
        if source is not None:
            seen={normal(f['question']) for f in facts}
            with (root/'external-exam.jsonl').open('w',encoding='utf-8') as output:
                for config in ['ARC-Easy','ARC-Challenge']:
                    rows=load_split(source,config,'test',root)
                    metrics={'questions':0,'correct':0,'answered':0,'training_question_overlap':0,'clean_questions':0,'clean_correct':0,'uniform_guess_expected_correct':0.0}
                    for row in rows:
                        # No expected answerKey, provenance ID, or subject label goes to Rust.
                        result=client.send('ask',question=row['question'])['result']
                        labels=list(map(str,row['choices']['label']));texts=row['choices']['text']
                        matched=[i for i,t in enumerate(texts) if result['answer'] is not None and normal(t)==normal(result['answer'])]
                        chosen=labels[matched[0]] if len(matched)==1 else None
                        correct=chosen==str(row['answerKey']);overlap=normal(row['question']) in seen
                        metrics['questions']+=1;metrics['correct']+=correct;metrics['answered']+=chosen is not None
                        metrics['training_question_overlap']+=overlap;metrics['clean_questions']+=not overlap;metrics['clean_correct']+=correct and not overlap
                        metrics['uniform_guess_expected_correct']+=1.0/len(texts)
                        output.write(json.dumps({'config':config,'id':row['id'],'question':row['question'],'choices':row['choices'],'selected':chosen,
                            'expected':str(row['answerKey']),'correct':correct,'overlap':overlap,'result':result},ensure_ascii=False)+'\n')
                    external[config]=metrics
        after=client.send('checkpoint')['checkpoint'];after_control=control.send('checkpoint')['checkpoint']
        if after!=checkpoint or after_control!=checkpoint:raise RuntimeError('Examination mutated knowledge or controller')
        summary={'status':'COMPLETED_DEVELOPMENT','human_12_year_old_level_established':False,
            'teacher_numeric_examples':len(all_teacher),'external_training_qa_records':len(external_train),'total_teacher_records':len(all_teacher)+len(external_train),
            'school':client.send('school_report')['report'],'checks':totals,'external':external,
            'external_format':'Free answer mapped to choices by evaluator; choices not used by learner. Not an official standard ARC score.',
            'retrieval_check':{'seen_questions':len(recall),'correct':sum(r['correct'] for r in recall)},
            'frozen_checkpoint_sha256':frozen_hash,'checkpoint_unchanged_after_all_exams':True,
            'separate_process_restore':True,'learned_fact_memory_is_not_reasoning':True,
            'selected_examples':[r for r in records if r['category']=='new_numbers'][::128][:12],
            'unassessed':['general reading comprehension','causal experiments','open-ended planning','social reasoning','visual object grounding','human age norms']}
        write_json(root/'summary.json',summary);write_json(root/'seen-retrieval.json',recall)
        print(json.dumps({k:v for k,v in summary.items() if k!='selected_examples'},ensure_ascii=False,indent=2))
    finally:client.close();control.close();cold.close()
if __name__=='__main__':main()
