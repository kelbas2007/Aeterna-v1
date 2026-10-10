// DEVELOPMENTAL-RELATION-1: binding a factual appearance from the
// beginning of an episode, then comparing later visible candidates.
// This augments the SINGLE learned actor's state representation. It
// NEVER selects a motor or provides a prewritten route.
// Sensor structure is declared (regular categorical channels), but
// no object-code meanings, correct actions, maps or target branches exist.
const RELATION_FEATURE_DIM:usize=96;
const RELATION_ACTOR_DIM:usize=GENERAL_ACTOR_DIM+RELATION_FEATURE_DIM;

#[derive(Debug,Clone)]
struct PhaseRelationalWorkspace {
    width:usize,
    height:usize,
    channels:usize,
    bits:usize,
    identity_channels:usize,
    synapse:usize,
    // Episode-local remembered observational content; RESET every
    // actual episode and excluded from learned model fingerprint.
    subject:Option<Vec<u8>>,
    first_view_had_subject:bool,
    seen_frames:u64,
    observed_matches:u64,
    observed_contrasts:u64,
}
impl PhaseRelationalWorkspace{
    fn raw_dim(&self)->usize {
        self.width*self.height*self.channels*self.bits
    }
    fn signature(&self,raw:&[f32],idx:usize)->Option<Vec<u8>>{
        if raw.len()!=self.raw_dim() || idx>=self.width*self.height {
            return None;
        }
        let mut values=Vec::with_capacity(self.identity_channels);
        let stride=self.channels*self.bits;
        for channel in 0..self.identity_channels {
            let mut value=0u8;
            for bit in 0..self.bits{
                let v=raw[idx*stride+channel*self.bits+bit];
                if v==1.0{value|=1<<bit;}
                else if v!=0.0{return None;}
            }
            values.push(value);
        }
        Some(values)
    }
    fn candidates(&self,raw:&[f32])->Option<Vec<(usize,Vec<u8>)>>{
        if raw.len()!=self.raw_dim(){return None;}
        let body_index=(self.width/2)*self.height+self.height-1;
        let mut all=Vec::new();
        for tile in 0..self.width*self.height {
            if tile==body_index {continue;}
            let signature=self.signature(raw,tile)?;
            if signature.iter().all(|&x|x==0){continue;}
            all.push((tile,signature));
        }
        // A candidate foreground object appears in a small number of
        // viewed cells, unlike the repeated background. This is an
        // appearance-based attention heuristic, NOT a trained RGB object
        // detector, fixed list of types, or assignment of correct target.
        Some(all.iter().filter(|(_,s)|all.iter()
            .filter(|(_,other)|other==s).count()<=2)
            .cloned().collect())
    }
    fn start(&mut self,raw:&[f32])->bool{
        let Some(candidates)=self.candidates(raw) else{return false;};
        self.subject=None;
        self.first_view_had_subject=false;
        self.seen_frames=1;
        self.observed_matches=0;
        self.observed_contrasts=0;
        let mut unique=Vec::<Vec<u8>>::new();
        for (_,sig) in &candidates {
            if !unique.contains(sig){unique.push(sig.clone());}
        }
        if unique.len()==1 {
            self.subject=unique.into_iter().next();
            self.first_view_had_subject=true;
        }
        true
    }
    fn next(&mut self,raw:&[f32])->bool{
        let Some(candidates)=self.candidates(raw) else{return false;};
        self.seen_frames+=1;
        if let Some(subject)=self.subject.as_ref(){
            self.observed_matches+=candidates.iter()
                .filter(|(_,s)|s==subject).count() as u64;
            self.observed_contrasts+=candidates.iter()
                .filter(|(_,s)|s!=subject).count() as u64;
        }
        true
    }
    fn features(&self,raw:&[f32],physical:f32)->Option<Vec<f32>>{
        let candidates=self.candidates(raw)?;
        let mut out=vec![0.0f32;RELATION_FEATURE_DIM];
        if physical<=1e-8 {return Some(out);}
        let Some(subject)=self.subject.as_ref() else{return Some(out);};
        out[0]=physical; // An actual observed and retained event exists.
        let mut same=0usize;
        let mut other=0usize;
        for (tile,signature) in candidates{
            let relation=if &signature==subject {same+=1;1u64}
                else {other+=1;2u64};
            // Relation+egocentric location is a GENERAL signed feature
            // available to an ordinary learned motor actor. The feature
            // does not encode the identity of any MiniGrid category.
            let mut hash=(tile as u64).wrapping_mul(0x9E3779B97F4A7C15)
                ^relation.wrapping_mul(0xD6E8FEB86659FD93);
            hash^=hash>>30;
            hash=hash.wrapping_mul(0xBF58476D1CE4E5B9);
            hash^=hash>>27;
            let i=4+(hash as usize)%(RELATION_FEATURE_DIM-4);
            out[i]+=physical;
        }
        out[1]=(same as f32).min(3.0)*physical;
        out[2]=(other as f32).min(3.0)*physical;
        out[3]=if same==0 {physical}else{0.0};
        Some(out)
    }
}

impl EvoPhase {
    /// No known 'key', 'ball' or match semantics. The only built-in
    /// scaffold is identity comparison of rare visual codes across time.
    pub fn enable_phase_native_relational_workspace(
        &mut self,width:usize,height:usize,channels:usize,bits:usize,
        identity_channels:usize,
    )->bool{
        if width==0||height==0||channels==0||bits==0||bits>8
            ||identity_channels==0||identity_channels>channels
            ||width.checked_mul(height)
                .and_then(|x|x.checked_mul(channels))
                .and_then(|x|x.checked_mul(bits))
                !=Some(self.config.sensory_cells) {
            return false;
        }
        let ready=self.phase_native.as_ref().and_then(|n|
            n.general_policy.as_ref()).is_some_and(|m|
            m.developmental_memory && m.relational_workspace.is_none());
        if !ready || self.config.dormant_cells==0 {return false;}
        let Some(cell)=self.dormant_range()
            .find(|&c|!self.cells[c].recruited) else{return false;};
        self.cells[cell].recruited=true;
        let synapse=self.native_synapse(0,cell);
        let syn=&mut self.synapses[synapse];
        syn.weight=0.6;
        syn.confidence=1.0;
        syn.eligibility=1.0;
        syn.phase_offset=wrap_phase(
            self.cells[syn.to].phase-self.cells[syn.from].phase
        );
        let model=self.phase_native.as_mut().unwrap().general_policy
            .as_mut().unwrap();
        for row in &mut model.weights {
            row.resize(RELATION_ACTOR_DIM,0.0);
        }
        for row in &mut model.eligibility {
            row.resize(RELATION_ACTOR_DIM,0.0);
        }
        model.relational_workspace=Some(PhaseRelationalWorkspace{
            width,height,channels,bits,identity_channels,synapse,
            subject:None,first_view_had_subject:false,
            seen_frames:0,observed_matches:0,observed_contrasts:0,
        });
        true
    }
    pub fn is_phase_native_relational_synapse(&self,index:usize)->bool{
        self.phase_native.as_ref().and_then(|n|n.general_policy.as_ref())
            .and_then(|p|p.relational_workspace.as_ref())
            .is_some_and(|m|m.synapse==index)
    }
    pub fn phase_native_relational_link(&self)->Option<usize>{
        self.phase_native.as_ref()?.general_policy.as_ref()?
            .relational_workspace.as_ref().map(|m|m.synapse)
    }
    pub fn phase_native_relational_held_subject(&self)->bool{
        self.phase_native.as_ref().and_then(|n|n.general_policy.as_ref())
            .and_then(|m|m.relational_workspace.as_ref())
            .is_some_and(|m|m.subject.is_some())
    }
    pub fn phase_native_relational_subject_frames(&self)->u64{
        self.phase_native.as_ref().and_then(|n|n.general_policy.as_ref())
            .and_then(|m|m.relational_workspace.as_ref())
            .map_or(0,|m|m.seen_frames)
    }
    pub fn phase_native_relational_readout(
        &self,raw:&[f32]
    )->Option<Vec<f32>>{
        let n=self.phase_native.as_ref()?;
        let m=n.general_policy.as_ref()?.relational_workspace.as_ref()?;
        let gate=conductance(&self.cells,&self.synapses[m.synapse],
            n.config.coherence_floor);
        m.features(raw,gate)
    }
    pub fn phase_native_relational_initial(&mut self,raw:&[f32])->bool{
        let Some(model)=self.phase_native.as_mut()
            .and_then(|n|n.general_policy.as_mut()) else{return false;};
        let Some(work)=model.relational_workspace.as_mut() else{return false;};
        work.start(raw)
    }
    pub fn phase_native_relational_factual_post(&mut self,raw:&[f32])->bool{
        let Some(model)=self.phase_native.as_mut()
            .and_then(|n|n.general_policy.as_mut()) else{return false;};
        let Some(work)=model.relational_workspace.as_mut() else{return false;};
        work.next(raw)
    }
}
