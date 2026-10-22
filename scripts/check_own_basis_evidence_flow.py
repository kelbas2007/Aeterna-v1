"""Compare two native readouts of the exact same frozen learned organism.

This diagnoses the knowledge/action bridge. It is not a learning-algorithm
comparison. Neither fork receives teaching or correctness feedback.
"""
import argparse, hashlib, json
from pathlib import Path
from external5_transport import Native
from learning_world_stream import read_checkpoint, save_checkpoint, source_hash
from own_basis_lifetime import load, score


def run(a):
    output=Path(a.output);output.mkdir(parents=True,exist_ok=False)
    checkpoint=read_checkpoint(a.checkpoint)
    assert json.loads(checkpoint)['cognition']['freeze_controller']
    save_checkpoint(output/'starting-organism.json.gz',checkpoint)
    result={'scope':'same frozen learned organism; only native evidence-to-action path differs',
            'initial_checkpoint_sha256':hashlib.sha256(checkpoint.encode()).hexdigest(),
            'correctness_feedback_to_organism':False,'teaching_events_added':0,'arms':{}}
    for name,binary in [('previous',a.previous_binary),('current',a.binary)]:
        agent=Native(binary,output/f'{name}-life.jsonl.gz','organism',checkpoint=checkpoint)
        try:
            before=agent.command({'op':'basis_report'})['report']['learned_fingerprint']
            scores={}
            for task in ['fashion','har','mnist']:
                offset=(2947 if task=='har' else 10000)-a.probe
                xs,ys,_=load(Path(a.data),task,'test',offset,a.probe)
                scores[task]=score(agent,task,xs,ys)
            after=agent.command({'op':'basis_report'})['report']['learned_fingerprint']
            assert before==after
            final=agent.checkpoint();save_checkpoint(output/f'{name}-organism.json.gz',final)
            result['arms'][name]={'binary_sha256':source_hash(binary),'scores':scores,
                                  'learned_fingerprint':before,'learned_state_unchanged':True}
        finally:agent.close()
    before=result['arms']['previous'];after=result['arms']['current']
    assert before['learned_fingerprint']==after['learned_fingerprint']
    result['comparison']={}
    for task in ['fashion','har','mnist']:
        p=before['scores'][task];q=after['scores'][task]
        assert p['readout_predictions']==q['readout_predictions']
        assert p['targets']==q['targets']
        assert q['readout_u1_disagreements']==0
        result['comparison'][task]={'samples':a.probe,'previous_u1_correct':p['native_u1']['correct'],
            'current_u1_correct':q['native_u1']['correct'],'same_readout_predictions':True,
            'current_readout_u1_disagreements':0}
    (output/'results.json').write_text(json.dumps(result,indent=2)+'\n')
    return result['comparison']


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for key in ['binary','previous-binary','checkpoint','data','output']:p.add_argument('--'+key,required=True)
    p.add_argument('--probe',type=int,default=256)
    a=p.parse_args()
    if not 32<=a.probe<=512:p.error('probe range is 32..512')
    print(json.dumps(run(a),indent=2))
