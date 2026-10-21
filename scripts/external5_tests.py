"""Format and observation-boundary regressions using tiny generated fixtures."""
import gzip,struct,tempfile,unittest,zipfile,json,hashlib,copy
from unittest.mock import patch
from pathlib import Path
import numpy as np
from external5_sensors import load_fashion,load_har,SENSOR_NAMES,MatchedKNN,blank,acquire
from external5_arc import targets
from run_external_continual5 import verify_data_files,dependency_versions,PINNED_VERSIONS
from audit_external5_data import audit,audit_trace
import prepare_external5_data as prepare

class ExternalInputs(unittest.TestCase):
    def sensory_trace_fixture(self):
        x=np.stack([np.zeros((28,28),dtype=np.uint8),np.full((28,28),17,dtype=np.uint8)])
        datasets={('fashion',False):(x,np.array([0,1]),None)}
        rows=[];seen=set();sequence=[('fashion',i,False) for i in range(2)]
        def ref(frame):
            key=hashlib.sha256(json.dumps(frame,separators=(',',':'),allow_nan=False).encode()).hexdigest()
            if key not in seen:rows.append({'kind':'sensor_frame','id':key,'frame':frame});seen.add(key)
            return key
        for i in range(2):
            current=blank('fashion');start=ref(current)
            rows.append({'kind':'restart','frame_ref':start,'metadata':{'dataset':'fashion','index':i,'training':False}})
            for step,action in enumerate([0,i+1]):
                available=[False]*16
                for motor in ([0] if step==0 else [1,2]):available[motor]=True
                rows.append({'kind':'intent','index':2*i+step,'action':action,'remaining_actions':2-step,
                    'offers':[{'available':v,'max_cost':1} for v in available],'pre_ref':ref(current)})
                if step==0:current=acquire(current,0,x[i])
                post=ref(current)
                rows.append({'kind':'actual_post','index':2*i+step,'action':action,'frame_ref':post,'cost':1,'reward':float(step==1)})
                rows.append({'kind':'commit','index':2*i+step,'step':{'action':action}})
            rows.append({'kind':'graded_episode','dataset':'fashion','index':i,'training':False,
                         'prediction':i,'target_after_response':i,'measurements':1})
        return rows,{'fashion':([0],[1,2])},datasets,sequence
    def run_trace_audit_fixture(self,rows,maps,datasets,sequence):
        with tempfile.TemporaryDirectory() as folder:
            path=Path(folder)/'trace.jsonl.gz'
            with gzip.open(path,'wt') as stream:
                for row in rows:stream.write(json.dumps(row)+'\n')
            return audit_trace(path,maps,datasets,sequence,'full')
    def test_serialized_sensory_audit_accepts_exact_coverage_and_actions(self):
        result=self.run_trace_audit_fixture(*self.sensory_trace_fixture())
        self.assertEqual(result,{'status':'PASS','actual_actions':4,'graded_windows':2})
    def test_serialized_sensory_audit_rejects_invalid_actions_and_forged_coverage(self):
        original,maps,datasets,sequence=self.sensory_trace_fixture()
        for defect,error in [('unavailable','unavailable'),('duplicate_grade','completed active sample'),
                             ('index_gap','sample sequence'),('missing_phase','sample sequence'),
                             ('deadline','deadline'),('missing_commit','missing native factual commit')]:
            with self.subTest(defect=defect):
                rows=copy.deepcopy(original);expected=list(sequence)
                if defect=='unavailable':next(r for r in rows if r['kind']=='intent')['action']=1
                elif defect=='duplicate_grade':
                    at=next(i for i,r in enumerate(rows) if r['kind']=='graded_episode');rows.insert(at+1,copy.deepcopy(rows[at]))
                elif defect=='index_gap':next(r for r in rows if r['kind']=='restart' and r['metadata']['index']==1)['metadata']['index']=2
                elif defect=='missing_phase':expected[1]=('har',0,False)
                elif defect=='deadline':next(r for r in rows if r['kind']=='intent')['remaining_actions']=99
                elif defect=='missing_commit':rows.pop(next(i for i,r in enumerate(rows) if r['kind']=='commit'))
                with self.assertRaisesRegex(ValueError,error):self.run_trace_audit_fixture(rows,maps,datasets,expected)
    def test_idx_headers_never_become_pixels_or_feedback_labels(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);pixels=np.stack([np.zeros((28,28),dtype=np.uint8),np.full((28,28),17,dtype=np.uint8)])
            (root/'train-images-idx3-ubyte.gz').write_bytes(gzip.compress(struct.pack('>IIII',2051,2,28,28)+pixels.tobytes()))
            (root/'train-labels-idx1-ubyte.gz').write_bytes(gzip.compress(struct.pack('>II',2049,2)+bytes([3,7])))
            x,y,_=load_fashion(root,'train',1,1)
            self.assertTrue(np.all(x==17));self.assertEqual(y.tolist(),[7])
    def test_har_reads_raw_channels_and_keeps_subjects_outside_observation(self):
        with tempfile.TemporaryDirectory() as folder:
            with zipfile.ZipFile(Path(folder)/'uci-har.zip','w') as archive:
                for i,name in enumerate(SENSOR_NAMES):
                    archive.writestr(f'UCI HAR Dataset/train/Inertial Signals/{name}_train.txt',
                        ' '.join(['0']*128)+'\n'+' '.join([str(i+1)]*128)+'\n')
                archive.writestr('UCI HAR Dataset/train/y_train.txt','1\n6\n')
                archive.writestr('UCI HAR Dataset/train/subject_train.txt','3\n15\n')
            x,y,subjects=load_har(folder,'train',1,1)
            self.assertEqual(x.shape,(1,9,128));self.assertEqual(float(x[0,8,0]),9)
            self.assertEqual(y.tolist(),[5]);self.assertEqual(subjects.tolist(),[15])
            first,labels,people=load_har(folder,'train',0,1)
            self.assertEqual(first.shape,(1,9,128));self.assertEqual(labels.tolist(),[0]);self.assertEqual(people.tolist(),[3])
    def test_changed_archive_is_rejected_despite_an_unchanged_manifest(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);raw=b'original';path=root/'sample.bin';path.write_bytes(raw)
            (root/'manifest.json').write_text(json.dumps({'files':[{'name':path.name,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}]}))
            verify_data_files(root,{'sample.bin'})
            path.write_bytes(b'corrupt!')
            with self.assertRaisesRegex(ValueError,'SHA256 mismatch'):verify_data_files(root,{'sample.bin'})
    def test_har_requires_one_valid_subject_for_every_requested_window(self):
        for subjects in ['1\n','31\n2\n','1.5\n2\n']:
            with self.subTest(subjects=subjects),tempfile.TemporaryDirectory() as folder:
                with zipfile.ZipFile(Path(folder)/'uci-har.zip','w') as archive:
                    for name in SENSOR_NAMES:
                        archive.writestr(f'UCI HAR Dataset/train/Inertial Signals/{name}_train.txt',(' '.join(['0']*128)+'\n')*2)
                    archive.writestr('UCI HAR Dataset/train/y_train.txt','1\n6\n')
                    archive.writestr('UCI HAR Dataset/train/subject_train.txt',subjects)
                with self.assertRaises(ValueError):load_har(folder,'train',0,2)
    def test_cached_har_cannot_mint_a_trusted_manifest_for_changed_content(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);path=root/'uci-har.zip';raw=b'1\n'
            expected={'train/y_train.txt':{'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}}
            with zipfile.ZipFile(path,'w') as archive:archive.writestr('UCI HAR Dataset/train/y_train.txt',raw)
            self.assertEqual(prepare.verify_har_archive(path,expected),1)
            with zipfile.ZipFile(path,'w') as archive:archive.writestr('UCI HAR Dataset/train/y_train.txt',b'2\n')
            with patch.object(prepare,'FASHION',{}),patch.object(prepare,'pinned_har_files',return_value=expected),patch('sys.argv',['prepare','--output',folder,'--har-mirrors']):
                with self.assertRaisesRegex(ValueError,'pinned HAR content mismatch'):prepare.main()
            self.assertFalse((root/'manifest.json').exists())
    def test_scoring_rejects_dependency_drift_while_development_records_it(self):
        with patch('run_external_continual5.importlib.metadata.version',side_effect=PINNED_VERSIONS.__getitem__):
            self.assertEqual(dependency_versions(True),PINNED_VERSIONS)
        changed={**PINNED_VERSIONS,'minigrid':'0.0.0'}
        with patch('run_external_continual5.importlib.metadata.version',side_effect=changed.__getitem__):
            with self.assertRaisesRegex(RuntimeError,'dependency versions differ'):dependency_versions(True)
            self.assertEqual(dependency_versions(False),changed)
    def test_mixed_frozen_source_evidence_is_rejected_before_corpus_access(self):
        for field in ['source_commit','binary_sha256']:
            with self.subTest(field=field),tempfile.TemporaryDirectory() as folder:
                root=Path(folder);(root/'sensor-study').mkdir()
                frozen={'source_commit':'a'*40,'binary_sha256':'b'*64};other={**frozen,field:'c'*len(frozen[field])}
                (root/'source-freeze.json').write_text(json.dumps(frozen));(root/'sensor-study/commitment.json').write_text(json.dumps(other))
                with patch('audit_external5_data.verify_data_files') as verify:
                    with self.assertRaisesRegex(ValueError,'identity mismatch'):audit(root,root)
                    verify.assert_not_called()
    def test_matched_control_cannot_use_unmeasured_training_axes(self):
        model=MatchedKNN();unknown={'kind':'signals','namespace':2,'channels':[None]*9}
        observed={'kind':'signals','namespace':2,'channels':[[1.0,2.0,3.0]]+[None]*8}
        model.observe(observed);model.remember(unknown,3);model.remember(observed,11)
        self.assertEqual(model.predict(observed,[3,11]),11)
    def test_click_target_is_inside_a_visible_component_with_a_hole(self):
        pixels=[[0]*10 for _ in range(10)]
        for y in range(3,6):
            for x in range(3,6):
                if (y,x)!=(4,4):pixels[y][x]=7
        x,y=targets(pixels)[0];self.assertEqual(pixels[y][x],7)

if __name__=='__main__':unittest.main()
