// Included in education.rs. Grammar productions come only from validated lessons.
// A slot can contain another acquired computation, not just one literal number.
// Full token coverage, symbolic ambiguity and bounded search are fail-closed.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedPlan {
    pub expression: Expr,
    pub meaning: Expr,
    pub value: Option<i64>,
    pub execution_error: Option<String>,
    pub acquired_applications: usize,
    pub evidence: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistinguishingProbe {
    pub question: String,
    pub inputs: Vec<i64>,
    pub predicted_answers: Vec<Option<i64>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Understanding {
    pub status: String,
    pub plans: Vec<GroundedPlan>,
    pub numbers: Vec<i64>,
    pub unknown_terms: Vec<String>,
    pub distinguishing_probe: Option<DistinguishingProbe>,
    pub work: u64,
    pub complete_search: bool,
}
impl Understanding {
    fn empty(status: &str) -> Self {
        Self { status: status.into(), plans: vec![], numbers: vec![], unknown_terms: vec![],
            distinguishing_probe: None, work: 0, complete_search: true }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingLimits {pub work:u64,pub tokens:usize,pub alternatives_per_span:usize}
impl Default for GroundingLimits {
    fn default()->Self { Self {work:1_000_000,tokens:80,alternatives_per_span:16} }
}
#[derive(Clone)]
struct GToken {text:String, number:Option<usize>}
#[derive(Clone)]
enum GAtom {Word(String),Slot(usize)}
struct GRule {atoms:Vec<GAtom>, function:u64, arity:usize, source:String}
#[derive(Clone)]
struct GNode {expr:Expr, meaning:Expr, steps:usize, evidence:BTreeSet<String>}
struct Grounder<'a> {
    school:&'a School, tokens:Vec<GToken>, numbers:Vec<i64>, rules:Vec<GRule>,
    chart:BTreeMap<(usize,usize),Vec<GNode>>, meter:Meter, limits:GroundingLimits,
}
// Preserve grouping. No words, mathematical topics or synonyms are hardcoded.
fn glex(question:&str)->Result<(Vec<GToken>,Vec<i64>)> {
    if !text_ok(question){return Err(Error::InvalidData);}
    let cs:Vec<_>=question.to_lowercase().chars().collect();let mut out=Vec::new();
    let mut numbers=Vec::new();let mut i=0;let mut word=String::new();
    fn flush(word:&mut String,out:&mut Vec<GToken>) {
        if !word.is_empty(){out.push(GToken{text:std::mem::take(word),number:None});}
    }
    while i<cs.len() {
        let sign=(cs[i]=='-'||cs[i]=='−') && i+1<cs.len() && cs[i+1].is_ascii_digit()
            && (i==0||!cs[i-1].is_alphanumeric());
        if cs[i].is_ascii_digit() || sign {
            if !word.is_empty(){return Err(Error::InvalidData);}
            let mut n=String::new();if sign{n.push('-');i+=1;}
            while i<cs.len()&&cs[i].is_ascii_digit(){n.push(cs[i]);i+=1;}
            if i+1<cs.len()&&['.',','].contains(&cs[i])&&cs[i+1].is_ascii_digit(){return Err(Error::Type);}
            if i<cs.len()&&cs[i].is_alphabetic(){return Err(Error::InvalidData);}
            if numbers.len()>=24{return Err(Error::Capacity);}
            let value=n.parse::<i64>().map_err(|_|Error::Overflow)?;
            out.push(GToken{text:n,number:Some(numbers.len())});numbers.push(value);
        } else if cs[i].is_alphanumeric() {word.push(cs[i]);i+=1;}
        else {flush(&mut word,&mut out);if "()+-*/%=<>".contains(cs[i]){out.push(GToken{text:cs[i].to_string(),number:None});}i+=1;}
    }
    flush(&mut word,&mut out);let mut balance=0i32;
    for t in &out {if t.text=="("{balance+=1;}else if t.text==")"{balance-=1;}if balance<0{return Err(Error::Type);}}
    if balance!=0{return Err(Error::Type);}Ok((out,numbers))
}
fn grules(school:&School)->Result<Vec<GRule>> {
    let mut rules=Vec::new();
    for skill in &school.skills {
        let mut atoms=Vec::new();let mut slots=BTreeSet::new();let mut words=0;
        for w in skill.pattern.split_whitespace() {
            if let Some(j)=w.strip_prefix("slot").and_then(|s|s.parse::<usize>().ok()) {
                if j>=skill.arity || !slots.insert(j){return Err(Error::InvalidData);}
                atoms.push(GAtom::Slot(j));
            } else {atoms.push(GAtom::Word(w.into()));words+=1;}
        }
        // Prevent same-span epsilon/unary cycles. The numeric leaf already is identity.
        if words==0 || slots.len()!=skill.arity {continue;}
        rules.push(GRule{atoms,function:skill.function,arity:skill.arity,source:format!("validated_lesson:{}",skill.pattern)});
    }
    Ok(rules)
}
impl Grounder<'_> {
    fn put(&mut self,lo:usize,hi:usize,node:GNode)->Result<()> {
        self.meter.charge(1+node.meaning.nodes() as u64)?;
        if node.expr.nodes()>256 || node.meaning.nodes()>512{return Err(Error::Capacity);}
        let entries=self.chart.entry((lo,hi)).or_default();
        if let Some(old)=entries.iter_mut().find(|x|x.meaning==node.meaning) {
            if node.steps<old.steps {*old=node;}return Ok(());
        }
        if entries.len()>=self.limits.alternatives_per_span{return Err(Error::Capacity);}
        entries.push(node);Ok(())
    }
    fn match_rule(&mut self,rule:usize,part:usize,pos:usize,end:usize,
                  args:&mut Vec<Option<GNode>>,out:&mut Vec<GNode>)->Result<()> {
        self.meter.charge(1)?;
        let atoms_len=self.rules[rule].atoms.len();
        if part==atoms_len {
            if pos!=end{return Ok(());}
            let nodes:Vec<_>=args.iter().cloned().collect::<Option<Vec<_>>>().ok_or(Error::InvalidData)?;
            let expr=Expr::Call(self.rules[rule].function,nodes.iter().map(|n|n.expr.clone()).collect());
            // Expansion is symbolic, before current input values are substituted.
            self.meter.charge(expr.nodes() as u64)?;
            let meaning=expanded(&expr,&self.school.library,0)?;
            let mut evidence=BTreeSet::from([self.rules[rule].source.clone()]);
            let steps=1+nodes.iter().map(|n|n.steps).sum::<usize>();
            for n in nodes{evidence.extend(n.evidence);}
            out.push(GNode{expr,meaning,steps,evidence});return Ok(());
        }
        if pos>=end || end-pos<atoms_len-part{return Ok(());}
        match self.rules[rule].atoms[part].clone() {
            GAtom::Word(w)=>{
                if self.tokens[pos].number.is_none()&&self.tokens[pos].text==w {
                    self.match_rule(rule,part+1,pos+1,end,args,out)?;
                }
            },
            GAtom::Slot(j)=>{
                let stop=end-(atoms_len-part-1);
                for k in pos+1..=stop {
                    self.meter.charge(1)?;
                    let next=self.chart.get(&(pos,k)).cloned().unwrap_or_default();
                    for node in next {args[j]=Some(node);self.match_rule(rule,part+1,k,end,args,out)?;}
                }
                args[j]=None;
            }
        }Ok(())
    }
    fn parse(&mut self)->Result<()> {
        let n=self.tokens.len();
        for length in 1..=n {for lo in 0..=n-length {
            let hi=lo+length;
            if length==1 {
                if let Some(j)=self.tokens[lo].number {
                    self.put(lo,hi,GNode{expr:Expr::Input(j),meaning:Expr::Input(j),steps:0,evidence:BTreeSet::new()})?;
                }
            }
            if length>=3 && self.tokens[lo].text=="("&&self.tokens[hi-1].text==")" {
                for node in self.chart.get(&(lo+1,hi-1)).cloned().unwrap_or_default(){self.put(lo,hi,node)?;}
            }
            for r in 0..self.rules.len() {
                self.meter.charge(1)?;
                if self.rules[r].atoms.len()>length{continue;}
                let mut out=Vec::new();let mut args=vec![None;self.rules[r].arity];
                self.match_rule(r,0,lo,hi,&mut args,&mut out)?;
                for node in out{self.put(lo,hi,node)?;}
            }
        }}Ok(())
    }
}
impl School {
    pub fn enable_composition(&mut self)->Result<()> {
        if self.frozen{return Err(Error::Busy);}self.composition=true;Ok(())
    }
    pub fn composition_enabled(&self)->bool {self.composition}
    pub fn understand(&self,question:&str,limits:GroundingLimits)->Result<Understanding> {
        if limits.work<1 || limits.work>5_000_000 || !(1..=80).contains(&limits.tokens)
            || !(1..=32).contains(&limits.alternatives_per_span){return Err(Error::InvalidData);}
        let (tokens,numbers)=match glex(question) {
            Ok(v)=>v,Err(e)=>return Ok(Understanding::empty(&format!("unsupported_or_invalid_input:{e}")))
        };
        if tokens.len()>limits.tokens {let mut r=Understanding::empty("comprehension_budget_exhausted");r.complete_search=false;return Ok(r);}
        let rules=grules(self)?;
        let vocabulary:BTreeSet<_>=rules.iter().flat_map(|r|r.atoms.iter().filter_map(|a|match a{GAtom::Word(w)=>Some(w.clone()),_=>None})).collect();
        let unknown_terms:Vec<_>=tokens.iter().filter(|t|t.number.is_none()&&!vocabulary.contains(&t.text)&&t.text!="("&&t.text!=")")
            .map(|t|t.text.clone()).collect::<BTreeSet<_>>().into_iter().collect();
        let mut g=Grounder{school:self,tokens,numbers,rules,chart:BTreeMap::new(),meter:Meter::new(limits.work),limits};
        let parse=g.parse();
        let complete_search=parse.is_ok();
        let raw=g.chart.get(&(0,g.tokens.len())).cloned().unwrap_or_default();
        let mut plans=Vec::new();
        for node in raw {
            // Arithmetic failure does not silently remove an alternative interpretation.
            let execution=run(&node.expr,&g.numbers,&self.library,&mut g.meter);
            let value=execution.as_ref().ok().copied();let execution_error=execution.err().map(|e|e.to_string());
            plans.push(GroundedPlan{expression:node.expr,meaning:node.meaning,value,execution_error,
                acquired_applications:node.steps,evidence:node.evidence.into_iter().collect()});
        }
        let mut complete_search=complete_search;
        if g.meter.remaining()==0 {complete_search=false;}
        let status=if !complete_search {"comprehension_budget_exhausted"}
            else if plans.is_empty(){if unknown_terms.is_empty(){"missing_arguments_or_unlearned_structure"}else{"unlearned_language"}}
            else if plans.len()>1{"ambiguous_program_meaning"}
            else if plans[0].value.is_none(){"invalid_arithmetic_domain"}
            else if plans[0].acquired_applications==0{"literal_only"}else{"grounded_composition"};
        let mut probe=None;
        if complete_search&&plans.len()>1 {
            let mut rng=0x49b79_u64;
            for _ in 0..64 {
                let inputs:Vec<_>=(0..g.numbers.len()).map(|_|{rng=rng.wrapping_mul(6364136223846793005).wrapping_add(1);2+((rng>>32)%19) as i64}).collect();
                let values:Vec<_>=plans.iter().map(|p|run(&p.expression,&inputs,&self.library,&mut g.meter).ok()).collect();
                if values.iter().all(Option::is_some)&&values.iter().any(|x|x!=&values[0]) {
                    let text=g.tokens.iter().map(|t|t.number.map(|j|inputs[j].to_string()).unwrap_or_else(||t.text.clone())).collect::<Vec<_>>().join(" ");
                    probe=Some(DistinguishingProbe{question:text,inputs,predicted_answers:values});break;
                }
            }
        }
        Ok(Understanding{status:status.into(),plans,numbers:g.numbers,unknown_terms,distinguishing_probe:probe,work:g.meter.spent,complete_search})
    }
    pub(crate) fn composed_proposals(&self,question:&str)->Result<Vec<Proposal>> {
        let u=self.understand(question,GroundingLimits::default())?;
        if u.status=="grounded_composition" || u.status=="literal_only" {
            let p=&u.plans[0];
            return Ok(vec![Proposal{fields:[0.5,0.0,0.0,0.95,1.0/(1.0+u.work as f32)],
                answer:SchoolAnswer{answer:p.value.map(|x|x.to_string()),method:"composed_induced_vm".into(),
                    reason:format!("complete_symbolic_parse:{}_acquired_applications",p.acquired_applications),
                    evidence:p.evidence.clone(),derivation:Some(p.expression.clone()),bound_numbers:u.numbers,
                    work:u.work,support:p.acquired_applications,controller_score:None}}]);
        }
        let mut answer=SchoolAnswer::unknown(&u.status);answer.work=u.work;answer.bound_numbers=u.numbers;
        answer.evidence=u.unknown_terms.iter().map(|w|format!("unlearned_term:{w}")).collect();
        Ok(vec![Proposal{fields:[0.0;5],answer}])
    }
}
fn composition_disabled(v:&bool)->bool{!*v}
impl School {
    /// One answer to a model-generated distinguishing question. This narrows
    /// this interpretation only; it never trains or mutates the frozen library.
    pub fn clarify(&self,question:&str,reply:i64)->Result<SchoolAnswer> {
        let u=self.understand(question,GroundingLimits::default())?;
        if !u.complete_search || u.status!="ambiguous_program_meaning" {
            return Ok(SchoolAnswer::unknown("no_pending_semantic_ambiguity"));
        }
        let Some(probe)=u.distinguishing_probe else{return Ok(SchoolAnswer::unknown("no_distinguishing_question_within_budget"));};
        let survivors:Vec<_>=u.plans.iter().zip(&probe.predicted_answers).filter(|(_,v)|**v==Some(reply)).map(|(p,_)|p).collect();
        if survivors.len()!=1 {return Ok(SchoolAnswer::unknown(if survivors.is_empty(){"clarification_contradicts_all_candidates"}else{"clarification_still_ambiguous"}));}
        let p=survivors[0];
        if p.value.is_none(){return Ok(SchoolAnswer::unknown("invalid_arithmetic_domain"));}
        let mut evidence=p.evidence.clone();evidence.push(format!("episodic_clarification:{} => {}",probe.question,reply));
        Ok(SchoolAnswer{answer:p.value.map(|v|v.to_string()),method:"clarified_composed_vm".into(),
            reason:"one_symbolic_interpretation_remains_not_new_learning".into(),evidence,derivation:Some(p.expression.clone()),
            bound_numbers:u.numbers,work:u.work,support:p.acquired_applications,controller_score:None})
    }
}
