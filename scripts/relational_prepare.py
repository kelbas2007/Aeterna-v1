#!/usr/bin/env python3
"""Install learned REL-1 state, check TRAINING parity, freeze. No test file is read."""
from __future__ import annotations
import argparse, hashlib, json
from pathlib import Path
from word_problem_external import Client, install, dump, sha


def preserve(before,after):
    a=before['cognition'];b=after['cognition']
    for key in ['current','archived','nonce','operations','total_external_cost','max_external_cost']:
        assert a[key]==b[key],('world state changed',key)
    aa=json.loads(bytes(before['carrier']));bb=json.loads(bytes(after['carrier']))
    for key in ['cells','synapses','observations','tick']:
        assert aa[key]==bb[key],('controller state changed',key)
    old=a.get('education') or {};new=b.get('education') or {}
    for key,value in old.items():
        if key!='frozen':assert value==new[key],('old knowledge changed',key)


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--learned',type=Path,required=True)
    ap.add_argument('--prior',type=Path,required=True);ap.add_argument('--output',type=Path,required=True)
    args=ap.parse_args();args.binary=args.binary.resolve();root=args.output;root.mkdir(parents=True,exist_ok=True)
    model=json.loads((args.learned/'relational-model.json').read_text(encoding='utf-8'))
    c=Client(args.binary,root/'prepare-native.log')
    c.request({'op':'load','checkpoint':args.prior.read_text(encoding='utf-8')})
    before=json.loads(c.request({'op':'checkpoint'})['checkpoint'])
    c.request({'op':'begin_education_extension'})
    c.request({'op':'install_quantity_relations','model':model});c.request({'op':'freeze'})
    checkpoint=c.request({'op':'checkpoint'})['checkpoint'];after=json.loads(checkpoint);preserve(before,after)
    dest=root/'learned-relations.json';dest.write_text(checkpoint,encoding='utf-8')
    fixtures=json.loads((args.learned/'parity.json').read_text(encoding='utf-8'));cache={};checks=0
    for f in fixtures:
        actual=c.request({'op':'inspect_relational','question':f['question'],'a':f['a'],'b':f['b']})['features']
        assert actual==f['features'],('features',f['question'],f['a'],f['b'],set(actual.items())^set(f['features'].items()))
        if f['question'] not in cache:cache[f['question']]=c.request({'op':'ask_relational','question':f['question']})['result']
        result=cache[f['question']]
        scores=result['unary_scores'][f['a']] if f['b'] is None else next(r['scores'] for r in result['relations'] if r['first']==f['a'] and r['second']==f['b'])
        assert len(scores)==len(f['scores'])
        for a,b in zip(scores,f['scores']):assert abs(a-b)<1e-8,('score',a,b,f['question'])
        checks+=len(scores)
    from relational_train import chart,expr,prev
    import numpy as np
    for question,result in cache.items():
        ts,values=prev.ground_raw(question)
        pair=np.array([r['scores'] for r in result['relations']])
        trial={'values':list(map(float,values))}
        reference=chart(trial,[np.array(result['unary_scores']),pair],w=model['relevance_weight'],kbest=model['beam_per_root'],max_used=model['max_used'])
        if reference:
            assert expr(reference[0][1])==result['interpretations'][0]['program'],('structured decoding mismatch',question,reference[0],result['interpretations'][0])
            # Verify that the original public answer operation now uses the fix.
            default=c.request({'op':'ask_word_problem','question':question})['result']
            assert default['answer']==result['answer']
    end=c.request({'op':'checkpoint'})['checkpoint'];assert checkpoint==end;c.close()
    # A distinct checkpoint for the matched whole-program teacher-derivation arm.
    control=root/'whole-derivation-checkpoint.json'
    install(args.binary,args.learned/'whole-derivation-model.json',args.prior,control,root/'whole-prepare.log')
    # Actual previous code/data remain a separate, unchanged control.
    (root/'previous-checkpoint.json').write_bytes(args.prior.read_bytes())
    c=Client(args.binary,root/'separate-process.log');c.request({'op':'load','checkpoint':checkpoint})
    assert c.request({'op':'checkpoint'})['checkpoint']==checkpoint
    retained=c.request({'op':'ask','question':'Вычти из суммы 17 и 23 произведение 3 и 4.','mode':'ComputeOnly'})['result']
    assert retained['answer']=='28';c.close()
    source_files=['src/quantity_relations.rs','src/word_problems.rs','src/education.rs','src/education_runtime.rs','src/bin/aeterna-unified.rs','scripts/relational_train.py','scripts/relational_prepare.py','tests/quantity_relations.rs','Cargo.toml','Cargo.lock']
    source={p:sha(Path(p)) for p in source_files}
    freeze={'stage':'FROZEN_BEFORE_REL1_EXTERNAL_OUTCOMES','source':source,'binary_sha256':sha(args.binary),
        'prior_sha256':sha(args.prior),'checkpoint_sha256':sha(dest),'control_checkpoint_sha256':sha(control),
        'learned_model_sha256':sha(args.learned/'relational-model.json'),'whole_model_sha256':sha(args.learned/'whole-derivation-model.json'),
        'training_report_sha256':sha(args.learned/'training-report.json'),'fixtures':len(fixtures),'native_scores_matched':checks,'structured_decisions_matched':len(cache),
        'prior_world_and_knowledge_preserved':True,'checkpoint_unchanged_after_parity':True,'retained_russian_answer':retained['answer'],
        'external_training':False,'SVAMP_status':'already_exposed_regression','ASDiv_status':'reserved_nonoverlapping_complement',
        'training_equations_used':True,'no_gold_test_data_to_solver':True}
    dump(root/'freeze.json',freeze);print(json.dumps(freeze,indent=2),flush=True)


if __name__=='__main__':main()
