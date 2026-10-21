"""Whole-brain software replay with an extra read-only retention-entry export.

No new world actions or new performance examination. The original replay checks
every native action/commit and both final states. Checkpoint export is read-only.
"""
import argparse,gzip,hashlib,json,sys
from pathlib import Path

ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'scripts'))
from external5_transport import Native
import replay_organism_lifetime as replay_module


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--results',required=True);p.add_argument('--variant',required=True);p.add_argument('--output',required=True)
    args=p.parse_args();output=Path(args.output)
    captures=[]
    class CapturingNative(Native):
        def reset(self,frame,**metadata):
            if (metadata.get('phase')=='fashion' and metadata.get('source_offset')==57000
                and metadata.get('index')==0 and not metadata.get('corrective_retry')):
                raw=self.checkpoint().encode()
                gzip.open(output/'retention-entry.json.gz','wb').write(raw)
                captures.append({'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)})
            return super().reset(frame,**metadata)
    replay_module.Native=CapturingNative
    proof=replay_module.replay(args.results,args.variant,output)
    assert len(captures)==1
    proof['read_only_retention_entry']=captures[0]
    (output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
    print(json.dumps(proof,indent=2))


if __name__=='__main__':main()
