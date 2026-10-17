#!/usr/bin/env python3
"""Frozen REL-1 evaluation. Only raw question text enters the Rust subprocess.

SVAMP is an already-exposed regression set. ASDiv complement membership is
computed from text overlap alone, BEFORE answer parsing. All residual problems,
including non-scalar or unsupported ones, remain in the primary denominator.
"""
from __future__ import annotations
import argparse,collections,hashlib,json,math,re,time,urllib.request,xml.etree.ElementTree as ET
from fractions import Fraction
from pathlib import Path
from word_problem_external import Client,dump,sha,numberless,correct
ASDIV='https://raw.githubusercontent.com/chaochun/nlu-asdiv-dataset/883f90a9a65bf00304ba8f37423910fe743abc47/dataset/ASDiv.xml'
SVAMP='https://raw.githubusercontent.com/arkilpatel/SVAMP/78e727689e1c1bebfc4be39c446898e8e10b0518/SVAMP.json'

def download(url,path):
    with urllib.request.urlopen(url,timeout=60) as response:raw=response.read()
    path.write_bytes(raw)
    return {'url':url,'sha256':sha(path),'bytes':len(raw)}

def text_key(s):return ' '.join(re.findall(r'[a-z0-9]+',s.lower()))

def asdiv_complement(path,teacher):
    exact={text_key(x['question']) for x in teacher};masked={numberless(x['question']) for x in teacher}
    document=ET.fromstring(path.read_bytes());problems=document.findall('.//Problem')
    if not problems:raise ValueError('No ASDiv Problem elements')
    retained=[];excluded=[]
    for index,p in enumerate(problems):
        body=p.findtext('Body') or '';question=p.findtext('Question') or '';text=body.strip()+' '+question.strip()
        # Membership uses NO answer, formula, grade or solver result.
        reason='exact_training_overlap' if text_key(text) in exact else 'masked_training_overlap' if numberless(text) in masked else None
        if reason:excluded.append({'index':index,'id':p.get('ID'),'reason':reason});continue
        retained.append({'Body':body,'Question':question,'source_index':index,'source_id':p.get('ID'),
                         'Answer_raw':p.findtext('Answer') or '', 'Equation':p.findtext('Formula') or ''})
    return retained,{'original_count':len(problems),'excluded':excluded,'retained':len(retained),'membership_uses_labels':False}

def scalar_answer(text):
    # Unit parentheticals are labels, not alternative correct answers. Never
    # silently take the first of a list or a quotient from mixed-number answers.
    s=re.sub(r'\s*\([^)]*\)\s*$','',str(text)).strip()
    if not re.fullmatch(r'[+-]?\d+(?:,\d{3})*(?:\.\d+)?(?:/\d+)?',s):return None
    try:return str(Fraction(s.replace(',','')))
    except (ValueError,ZeroDivisionError):return None

def collect(binary,checkpoint,rows,path,method='relations'):
    client=Client(binary,path.with_suffix('.native.log'));client.request({'op':'load','checkpoint':checkpoint.read_text(encoding='utf-8')})
    before=sha(checkpoint);actual_before=hashlib.sha256(client.request({'op':'checkpoint'})['checkpoint'].encode()).hexdigest()
    if actual_before!=before:raise ValueError('Restored state differs before exam')
    start=time.perf_counter();records=[]
    with path.open('w',encoding='utf-8') as output:
        for i,row in enumerate(rows):
            question=row['Body'] if method=='body_only' else row['Body'].strip()+' '+row['Question'].strip()
            if method in ['relations','body_only']:request={'op':'ask_relational','question':question}
            elif method=='sg1':request={'op':'ask','question':question,'mode':'ComputeOnly'}
            else:request={'op':'ask_word_problem','question':question}
            prediction=client.request(request)['result']
            record={'index':i,'question':question,'prediction':prediction};records.append(record)
            output.write(json.dumps(record,ensure_ascii=False,separators=(',',':'))+'\n')
    after=hashlib.sha256(client.request({'op':'checkpoint'})['checkpoint'].encode()).hexdigest();client.close()
    if before!=after:raise ValueError('Exam modified frozen knowledge')
    return records,{'records':len(rows),'seconds':time.perf_counter()-start,'state_unchanged':True,'predictions_sha256':sha(path)}

def group_diagnostic(rows,predictions):
    groups=collections.defaultdict(list)
    for i,row in enumerate(rows):groups[numberless(row['Body'])].append(i)
    eligible=[ids for ids in groups.values() if len(ids)>1 and len({numberless(rows[i]['Question']) for i in ids})>1]
    results=[]
    for ids in eligible:
        programs=[json.dumps(predictions[i]['prediction'].get('interpretations',[{}])[0].get('program'),sort_keys=True) if predictions[i]['prediction'].get('interpretations') else None for i in ids]
        results.append({'indices':ids,'different_selected_programs':len(set(programs))>1})
    return {'normalization':'number-masked Body; at least two distinct number-masked Questions','groups':len(results),'details':results,
            'not_a_correctness_score':True,'caveat':'A changed program is sensitivity, not necessarily a correct answer.'}

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--frozen',type=Path,required=True)
    ap.add_argument('--learned',type=Path,required=True);ap.add_argument('--output',type=Path,required=True)
    ap.add_argument('--svamp',type=Path);ap.add_argument('--asdiv',type=Path)
    args=ap.parse_args();args.binary=args.binary.resolve();out=args.output;out.mkdir(parents=True,exist_ok=True)
    freeze=json.loads((args.frozen/'freeze.json').read_text(encoding='utf-8'))
    for name,key in [('learned-relations.json','checkpoint_sha256'),('whole-derivation-checkpoint.json','control_checkpoint_sha256'),('previous-checkpoint.json','prior_sha256')]:
        assert sha(args.frozen/name)==freeze[key]
    for name,key in [('relational-model.json','learned_model_sha256'),('whole-derivation-model.json','whole_model_sha256')]:assert sha(args.learned/name)==freeze[key]
    # Solver source may differ only in non-semantic platform builds. CI checks
    # source hashes separately; the actual inference binary is identified here.
    manifest={'source_freeze_sha256':sha(args.frozen/'freeze.json'),'binary_sha256':sha(args.binary),'datasets':{}}
    for name,url,given in [('SVAMP-original.json',SVAMP,args.svamp),('ASDiv-original.xml',ASDIV,args.asdiv)]:
        path=out/name
        if given:path.write_bytes(given.read_bytes());manifest['datasets'][name]={'url':url,'sha256':sha(path),'provided':True}
        else:manifest['datasets'][name]=download(url,path)
    dump(out/'manifest.json',manifest)
    svamp=json.loads((out/'SVAMP-original.json').read_text(encoding='utf-8'));assert len(svamp)==1000
    teacher=json.loads((args.learned/'teacher.json').read_text(encoding='utf-8'))
    asdiv,membership=asdiv_complement(out/'ASDiv-original.xml',teacher);dump(out/'ASDiv-membership.json',membership)
    dump(out/'ASDiv-complement.json',asdiv)
    final={}
    for dataset,rows in [('SVAMP-regression',svamp),('ASDiv-complement',asdiv)]:
        folder=out/dataset;folder.mkdir(exist_ok=True);all_predictions={};arms={}
        for method,checkpoint in [('relations','learned-relations.json'),('body_only','learned-relations.json'),('whole_derivations','whole-derivation-checkpoint.json'),('previous_ewp1','previous-checkpoint.json'),('sg1','learned-relations.json')]:
            predictions,trace=collect(args.binary,args.frozen/checkpoint,rows,folder/(method+'.jsonl'),method)
            all_predictions[method]=predictions;arms[method]=trace
        # First read expected answers only AFTER every arm's predictions exist.
        gold=[str(r['Answer']) if dataset=='SVAMP-regression' else scalar_answer(r['Answer_raw']) for r in rows]
        for method,predictions in all_predictions.items():
            matched=[correct(p['prediction'].get('answer'),a) if a is not None else False for p,a in zip(predictions,gold)]
            arms[method].update(correct=sum(matched),total=len(rows),refusals=sum(p['prediction'].get('answer') is None for p in predictions),
                                scalar_reference_count=sum(a is not None for a in gold),correct_indices=[i for i,b in enumerate(matched) if b])
        full=all_predictions['relations'];old=all_predictions['previous_ewp1'];whole=all_predictions['whole_derivations']
        # Same deterministic example selection regardless of how good the result is.
        examples={'first_new_successes':[],'first_failures':[]}
        for i,(row,p,oldp) in enumerate(zip(rows,full,old)):
            right=correct(p['prediction'].get('answer'),gold[i]) if gold[i] is not None else False
            record={'index':i,'question':p['question'],'prediction':p['prediction'],'reference':gold[i]}
            key='first_new_successes' if right and not correct(oldp['prediction'].get('answer'),gold[i]) else 'first_failures' if not right else None
            if key and len(examples[key])<6:examples[key].append(record)
        dump(folder/'examples.json',examples)
        final[dataset]={'arms':arms,'question_groups':group_diagnostic(rows,full),'exposed_before_this_turn':dataset=='SVAMP-regression'}
        print(dataset,{k:(v['correct'],v['total']) for k,v in arms.items()},flush=True)
    final['limitations']=['Teacher training equations were used this time.','Numeric accuracy is not semantic correctness.','No arbitrary human-level intelligence claim.','ASDiv complement excludes exact/masked training text, not every possible semantic near-duplicate.']
    dump(out/'summary.json',final)

if __name__=='__main__':main()
