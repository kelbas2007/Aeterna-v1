"""Verify that an exported organism resumes real learning without a new init.

Uses one previously experienced public training image. This is a software
continuation check, not a new recognition or intelligence benchmark.
"""
import argparse,gzip,hashlib,json
from pathlib import Path
from external5_transport import Native
from external5_sensors import load_fashion
from run_external_continual5 import motors,verify_data_files

def verify(binary,checkpoint,data,output):
    output=Path(output);output.mkdir(parents=True,exist_ok=False)
    verify_data_files(data)
    path=Path(checkpoint)
    saved=gzip.decompress(path.read_bytes()).decode() if path.suffix=='.gz' else path.read_text()
    x,y,_=load_fashion(data,'train',42000,1)
    frame={'kind':'image','namespace':1,'pixels':x[0].tolist()}
    _,answers=motors('fashion');calls=[]
    agent=Native(binary,output/'continuation.jsonl.gz','organism',frame,saved)
    try:
        before=agent.command({'op':'organism_report'})
        if before['organism']['controller_frozen']:raise AssertionError('checkpoint has frozen learning')
        agent.reset(frame,phase='fashion',index=0,software_continuation=True)
        def execute(action):
            calls.append(action)
            return {'frame':frame,'cost':1,'reward':float(answers.index(action)==int(y[0]))}
        _,post=agent.step([a in answers for a in range(16)],1,execute)
        agent.finish();after=agent.command({'op':'organism_report'})
        assert len(calls)==1
        assert after['organism']['total_external_cost']==before['organism']['total_external_cost']+1
        assert after['reward_memory']['factual_events']==before['reward_memory']['factual_events']+1
        assert after['reward_memory']['learned_events']>before['reward_memory']['learned_events']
        assert after['reward_memory']['learned_fingerprint']!=before['reward_memory']['learned_fingerprint']
        assert after['development']['episodes']==before['development']['episodes']+1
        assert after['development']['namespaces']==before['development']['namespaces']
        continued=agent.checkpoint()
        gzip.open(output/'continued-organism.json.gz','wt').write(continued)
        proof={'status':'PASS','purpose':'software continuation on an already experienced public image, not a new exam',
            'binary_sha256':hashlib.sha256(Path(binary).read_bytes()).hexdigest(),
            'input_checkpoint_sha256':hashlib.sha256(saved.encode()).hexdigest(),
            'before_events':before['reward_memory']['factual_events'],'after_events':after['reward_memory']['factual_events'],
            'before_episodes':before['development']['episodes'],'after_episodes':after['development']['episodes'],
            'protected_actions':len(calls),'actual_cost':post['cost'],'actual_reward':post['reward'],
            'retained_interfaces':after['development']['namespaces'],'learned_fingerprint_changed':True,
            'continued_checkpoint_sha256':hashlib.sha256(continued.encode()).hexdigest()}
        (output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
        return proof
    finally:agent.close()

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    for field in ('binary','checkpoint','data','output'):parser.add_argument('--'+field,required=True)
    args=parser.parse_args()
    print(json.dumps(verify(args.binary,args.checkpoint,args.data,args.output),indent=2))
