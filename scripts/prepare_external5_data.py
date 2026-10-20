"""Download official data without decoding any held-out record or answer."""
from __future__ import annotations
import argparse, hashlib, json, urllib.request, urllib.parse, zipfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

FASHION_COMMIT = 'b2617bb6d3ffa2e429640350f613e3291e10b141'
FASHION = {
    'train-images-idx3-ubyte.gz': '8d4fb7e6c68d591d4c3dfef9ec88bf0d',
    'train-labels-idx1-ubyte.gz': '25c81989df183df01b3e8a0aad5dffbe',
    't10k-images-idx3-ubyte.gz': 'bef4ecab320f06d8554ea6380940ec79',
    't10k-labels-idx1-ubyte.gz': 'bb300cfdad3c16e7a12a480ee83cd310',
}
HAR = 'https://archive.ics.uci.edu/static/public/240/human+activity+recognition+using+smartphones.zip'
HAR_MIRRORS = [
    ('schakraborty369/UCI-HAR-Dataset','af2d3fbe51b5c317e34701225d5420d0d884d5ce','data/'),
    ('babarbashir/UCI-HAR-Dataset','9f4dca7e33c616fe78e8a51f371c2d2e33d48b4b','UCI-HAR-Dataset/'),
]

def request(url):return urllib.request.urlopen(urllib.request.Request(url,headers={'User-Agent':'AETERNA-external-data/5'}),timeout=120)

def mirrored_har(root):
    """Cross-check two pinned mirrors at the byte level; do not decode records."""
    sources=[]
    for repo,commit,prefix in HAR_MIRRORS:
        with request(f'https://api.github.com/repos/{repo}/git/trees/{commit}?recursive=1') as r:tree=json.load(r)
        files={v['path'][len(prefix):]:v['sha'] for v in tree['tree']
               if v['type']=='blob' and v['path'].startswith(prefix)}
        sources.append(files)
    names=[f'{part}/{field}_{part}.txt' for part in ['train','test'] for field in ['y','subject']]
    names += [f'{part}/Inertial Signals/{sensor}_{axis}_{part}.txt'
              for part in ['train','test'] for sensor in ['body_acc','body_gyro','total_acc'] for axis in ['x','y','z']]
    for name in names:
        if sources[0].get(name)!=sources[1].get(name) or name not in sources[0]:
            raise ValueError(f'independent HAR mirrors disagree: {name}')
    cache=root/'har-mirror-files';cache.mkdir(exist_ok=True)
    def download(name):
        repo,commit,prefix=HAR_MIRRORS[0]
        url=f'https://raw.githubusercontent.com/{repo}/{commit}/'+urllib.parse.quote(prefix+name)
        path=cache/name;path.parent.mkdir(parents=True,exist_ok=True)
        if not path.exists():
            with request(url) as response:raw=response.read()
            path.write_bytes(raw)
        raw=path.read_bytes()
        blob=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
        if blob!=sources[0][name]:raise ValueError(f'HAR transport integrity failure: {name}')
        return {'path':name,'source_url':url,'bytes':len(raw),'git_blob_sha1':blob,
                'sha256':hashlib.sha256(raw).hexdigest(),'independent_mirrors_match':True}
    with ThreadPoolExecutor(3) as pool:files=list(pool.map(download,names))
    archive=root/'uci-har.zip'
    with zipfile.ZipFile(archive,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=6) as z:
        for name in names:z.write(cache/name,'UCI HAR Dataset/'+name)
    record={'transport':'two_pinned_matching_public_mirrors','official_origin':HAR,
            'sources':[{'repository':r,'commit':c,'prefix':p} for r,c,p in HAR_MIRRORS],
            'files':files,'note':'Archive repacked from raw inertial files and original partition metadata; no record decoded.'}
    (root/'har-mirror-manifest.json').write_text(json.dumps(record,indent=2)+'\n')
    print('HAR independent mirrors verified; original UCI host unavailable.',flush=True)

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--output',required=True);ap.add_argument('--har-mirrors',action='store_true');args=ap.parse_args()
    root=Path(args.output);root.mkdir(parents=True,exist_ok=True); manifest=[]
    jobs=[(n,f'https://raw.githubusercontent.com/zalandoresearch/fashion-mnist/{FASHION_COMMIT}/data/fashion/{n}',md5)
          for n,md5 in FASHION.items()]+[('uci-har.zip',HAR,None)]
    for name,url,expected_md5 in jobs:
        path=root/name
        if name=='uci-har.zip' and args.har_mirrors and not path.exists():mirrored_har(root)
        if not path.exists():
            temporary=path.with_suffix(path.suffix+'.partial')
            req=urllib.request.Request(url,headers={'User-Agent':'AETERNA-external-data/5'})
            with urllib.request.urlopen(req,timeout=120) as response,temporary.open('wb') as f:
                while block:=response.read(1024*1024):f.write(block)
            temporary.replace(path)
        raw=path.read_bytes();md5=hashlib.md5(raw).hexdigest()
        if expected_md5 and md5!=expected_md5:raise ValueError(f'official checksum mismatch: {name}')
        row={'name':name,'url':url,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),
             'official_md5':expected_md5,'md5_verified':bool(expected_md5)}
        if name=='uci-har.zip' and (root/'har-mirror-manifest.json').exists():
            row['transport']='repacked_matching_mirrors';row['source_manifest']='har-mirror-manifest.json'
        manifest.append(row);print(json.dumps(row),flush=True)
    (root/'manifest.json').write_text(json.dumps({'fashion_source_commit':FASHION_COMMIT,
        'held_out_decoded':False,'files':manifest},indent=2)+'\n')

if __name__=='__main__':main()
