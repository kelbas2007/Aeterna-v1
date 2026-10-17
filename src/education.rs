//! Teacher evidence -> learned lexical binding -> induced VM computation.
//! No language model, subject-specific dispatch, formulas or test answers.
//! This is controlled-text grounding and retrieval, not general language understanding.
use crate::construction::induction::{self,Example,InductionConfig};
use crate::construction::vm::*;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TeachingExample {pub question:String,pub answer:i64,pub source:String}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fact {pub question:String,pub answer:String,pub source:String}
#[derive(Debug,Clone,Serialize,Deserialize)]
struct Skill {pattern:String,arity:usize,function:u64,training:Vec<TeachingExample>,validation:Vec<TeachingExample>,work:u64}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct LessonReport {
    pub pattern:String,pub training:usize,pub validation:usize,pub validation_correct:usize,
    pub accepted:bool,pub work:u64,pub search_truncated:bool,pub function:Option<u64>,
    pub validation_outcomes:Vec<bool>,
    pub reason:String,pub expression:Option<Expr>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SchoolReport {pub teacher_examples:usize,pub acquired_skills:usize,pub library_functions:usize,
    pub fact_records:usize,pub failed_lessons:usize,pub training_work:u64,pub frozen:bool}
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub enum AnswerMode {All,RecallOnly,ComputeOnly}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SchoolAnswer {
    pub answer:Option<String>,pub method:String,pub reason:String,pub evidence:Vec<String>,
    pub derivation:Option<Expr>,pub bound_numbers:Vec<i64>,pub work:u64,pub support:usize,
    pub controller_score:Option<f32>,
}
impl SchoolAnswer {pub fn unknown(reason:&str)->Self {Self{answer:None,method:"abstain".into(),reason:reason.into(),
    evidence:vec![],derivation:None,bound_numbers:vec![],work:0,support:0,controller_score:None}}}
#[derive(Debug,Clone)]
pub(crate) struct Proposal {pub answer:SchoolAnswer,pub fields:[f32;5]}
#[derive(Debug,Default,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct School {
    library:Library,skills:Vec<Skill>,facts:Vec<Fact>,lessons:Vec<LessonReport>,frozen:bool,
    fact_index:BTreeMap<String,Vec<usize>>,terms:BTreeMap<String,Vec<usize>>,word_counts:Vec<usize>,
    #[serde(default,skip_serializing_if="composition_disabled")]
    composition:bool,
    #[serde(default,skip_serializing_if="Option::is_none")]
    word_problems:Option<crate::word_problems::WordProblemModel>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    quantity_relations:Option<crate::quantity_relations::RelationalModel>,
}
fn clean(s:&str)->String {
    s.to_lowercase().chars().map(|c|if c.is_alphanumeric() || "+-*/%=<>".contains(c){c}else{' '})
        .collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ")
}
fn text_ok(s:&str)->bool {!s.trim().is_empty() && s.len()<=4096 && !s.chars().any(|c|c=='\0')}
/// Generic integer tokenization. The vocabulary around the slots is LEARNED.
/// Decimals are rejected rather than misread as two integers.
pub fn ground(question:&str)->Result<(String,Vec<i64>)> {
    if !text_ok(question){return Err(Error::InvalidData);}
    let chars:Vec<_>=question.to_lowercase().chars().collect();let mut i=0;let mut text=String::new();let mut values=Vec::new();
    while i<chars.len() {
        let c=chars[i];
        let sign=(c=='-' || c=='−') && i+1<chars.len() && chars[i+1].is_ascii_digit()
            && (i==0 || !chars[i-1].is_alphanumeric());
        if c.is_ascii_digit() || sign {
            if i>0 && chars[i-1].is_alphabetic(){return Err(Error::InvalidData);}
            let mut n=String::new();if sign {n.push('-');i+=1;}
            while i<chars.len() && chars[i].is_ascii_digit(){n.push(chars[i]);i+=1;}
            if i+1<chars.len() && (chars[i]=='.'||chars[i]==',') && chars[i+1].is_ascii_digit(){return Err(Error::InvalidData);}
            let v=n.parse::<i64>().map_err(|_|Error::Overflow)?;if values.len()>=6{return Err(Error::Capacity);}
            text.push_str(&format!(" SLOT{} ",values.len()));values.push(v);
        } else {text.push(c);i+=1;}
    }
    Ok((clean(&text),values))
}
fn expanded(e:&Expr,lib:&Library,depth:usize)->Result<Expr> {
    if depth>32{return Err(Error::Capacity);}
    let mut error=None;
    let out=e.map(&mut |p|if let Expr::Call(id,args)=p {
        let result=(||{let f=lib.functions.iter().find(|f|f.id==*id).ok_or(Error::InvalidProgram)?;
            let body=f.body.map(&mut |a|if let Expr::Arg(i)=a {args.get(*i).cloned()} else {None});
            expanded(&body,lib,depth+1)})();
        match result {Ok(v)=>Some(v),Err(err)=>{error=Some(err);Some(p.clone())}}
    } else {None});
    match error {Some(e)=>Err(e),None=>Ok(canonical(&out))}
}
// Only exact algebraic identities; observational agreement is not enough.
fn canonical(e:&Expr)->Expr {
    match e {
        Expr::Bin(op,a,b)=>{let mut a=canonical(a);let mut b=canonical(b);
            if matches!(op,Op::Add|Op::Mul|Op::Xor|Op::Eq|Op::And|Op::Or) && a>b {std::mem::swap(&mut a,&mut b);}
            Expr::bin(*op,a,b)},
        Expr::AsInt(a)=>Expr::AsInt(Box::new(canonical(a))),
        Expr::Not(a)=>Expr::Not(Box::new(canonical(a))),
        Expr::If(a,b,c)=>Expr::If(Box::new(canonical(a)),Box::new(canonical(b)),Box::new(canonical(c))),
        _=>e.clone(),
    }
}
fn run(e:&Expr,input:&[i64],lib:&Library,meter:&mut Meter)->Result<i64> {
    e.eval(Context{input,action:0,state:&[],args:&[]},lib,meter)?.int()
}
impl School {
    pub fn freeze(&mut self){self.frozen=true;}
    pub fn is_frozen(&self)->bool{self.frozen}
    pub fn report(&self)->SchoolReport {
        SchoolReport {teacher_examples:self.lessons.iter().map(|x|x.training+x.validation).sum(),
            acquired_skills:self.skills.len(),library_functions:self.library.functions.len(),fact_records:self.facts.len(),
            failed_lessons:self.lessons.iter().filter(|x|!x.accepted).count(),training_work:self.lessons.iter().map(|x|x.work).sum(),frozen:self.frozen}
    }
    pub fn lessons(&self)->&[LessonReport]{&self.lessons}
    /// One declared lesson, with the last quarter held out before induction.
    /// No formula or task ID is accepted by this API. Failed validation is final
    /// for this lexical pattern, not a signal for fitting that validation set.
    pub fn teach(&mut self,examples:Vec<TeachingExample>)->Result<LessonReport> {
        if self.frozen{return Err(Error::Busy);}
        if !(24..=128).contains(&examples.len()) || self.lessons.len()>=128{return Err(Error::Capacity);}
        let mut parsed=Vec::new();let mut unique=BTreeSet::new();
        for e in &examples {
            if !text_ok(&e.source){return Err(Error::InvalidData);}
            let (key,input)=ground(&e.question)?;
            if input.is_empty() || !unique.insert(input.clone()){return Err(Error::InvalidData);}
            parsed.push((key,Example{input,output:e.answer}));
        }
        let pattern=parsed[0].0.clone();let arity=parsed[0].1.input.len();
        if parsed.iter().any(|(key,_)|key!=&pattern) || self.lessons.iter().any(|r|r.pattern==pattern){return Err(Error::Stale);}
        let split=examples.len()*3/4;
        // Require varied arguments; a single repeated example cannot certify a skill.
        if (0..arity).any(|i|parsed[..split].iter().map(|(_,r)|r.input[i]).collect::<BTreeSet<_>>().len()<3){return Err(Error::NoEvidence);}
        let training:Vec<_>=parsed[..split].iter().map(|(_,e)|e.clone()).collect();
        let fit=induction::infer(&training,&self.library,&InductionConfig::default())?;
        let mut report=LessonReport{pattern:pattern.clone(),training:split,validation:examples.len()-split,validation_correct:0,
            accepted:false,work:fit.work,search_truncated:fit.truncated,function:None,validation_outcomes:vec![],reason:"no_program_within_budget".into(),expression:fit.expression.clone()};
        if let Some(expression)=fit.expression {
            let mut meter=Meter::new(100_000);
            for (_,e) in &parsed[split..] {
                let correct=run(&expression,&e.input,&self.library,&mut meter)==Ok(e.output);
                report.validation_outcomes.push(correct);if correct{report.validation_correct+=1;}
            }
            report.work+=meter.spent;report.reason="prospective_validation_failed".into();
            if report.validation_correct==report.validation {
                let body=expression.map(&mut |e|if let Expr::Input(i)=e{Some(Expr::Arg(*i))}else{None});
                let flat=expanded(&body,&self.library,0)?;
                let reused=self.library.functions.iter().find(|f|f.arity==arity&&expanded(&f.body,&self.library,0)==Ok(flat.clone())).map(|f|f.id);
                let id=if let Some(id)=reused{id}else{
                    if self.library.functions.len()>=32{return Err(Error::Capacity);}
                    let id=fingerprint(&serde_json::to_vec(&(arity,&body)).map_err(|_|Error::InvalidData)?);
                    if self.library.functions.iter().any(|f|f.id==id){return Err(Error::InvalidProgram);}
                    let f=Function{id,arity,body};let mut candidate=self.library.clone();candidate.functions.push(f);candidate.validate(6)?;
                    self.library=candidate;id
                };
                report.accepted=true;report.function=Some(id);report.reason="validated_teacher_rule_not_general_language".into();
                self.skills.push(Skill{pattern,arity,function:id,training:examples[..split].to_vec(),validation:examples[split..].to_vec(),work:report.work});
            }
        }
        self.lessons.push(report.clone());Ok(report)
    }
    pub fn teach_facts(&mut self,facts:Vec<Fact>)->Result<usize> {
        if self.frozen{return Err(Error::Busy);}
        if self.facts.len()+facts.len()>8192 || facts.iter().any(|f|!text_ok(&f.question)||!text_ok(&f.answer)||!text_ok(&f.source)){return Err(Error::Capacity);}
        // Source statements are remembered, not asserted as infallible truth.
        let mut known:BTreeSet<_>=self.facts.iter().map(|f|(clean(&f.question),clean(&f.answer),f.source.clone())).collect();
        let mut inserted=0;
        for f in facts {if known.insert((clean(&f.question),clean(&f.answer),f.source.clone())){let key=clean(&f.question);let index=self.facts.len();
            self.fact_index.entry(key.clone()).or_default().push(index);
            let words:BTreeSet<_>=key.split_whitespace().map(str::to_string).collect();self.word_counts.push(words.len());
            for w in words{self.terms.entry(w).or_default().push(index);}
            self.facts.push(f);inserted+=1;}}
        Ok(inserted)
    }
    pub(crate) fn propose(&self,question:&str,mode:AnswerMode)->Result<Vec<Proposal>> {
        if !text_ok(question){return Err(Error::InvalidData);}
        let mut out=Vec::new();
        if mode!=AnswerMode::RecallOnly {
            if let Ok((key,input))=ground(question) {
                for s in self.skills.iter().filter(|s|s.pattern==key && s.arity==input.len()) {
                    let expr=Expr::Call(s.function,input.iter().copied().map(Expr::integer).collect());let mut meter=Meter::new(100_000);
                    if let Ok(v)=run(&expr,&[],&self.library,&mut meter) {
                        let mut evidence:Vec<_>=s.validation.iter().chain(&s.training).map(|e|e.source.clone()).collect();evidence.sort();evidence.dedup();evidence.truncate(8);
                        out.push(Proposal{fields:[0.5,0.0,0.0,0.95,1.0/(1.0+meter.spent as f32)],answer:SchoolAnswer{answer:Some(v.to_string()),
                            method:"induced_vm".into(),reason:"learned_numeric_binding".into(),evidence,derivation:Some(expr),bound_numbers:input.clone(),
                            work:meter.spent,support:s.training.len()+s.validation.len(),controller_score:None}});
                    }
                }
            }
        }
        if mode!=AnswerMode::ComputeOnly {
            let key=clean(question);
            let exact:Vec<_>=self.fact_index.get(&key).into_iter().flatten().filter_map(|&i|self.facts.get(i)).collect();
            if !exact.is_empty() {
                let answers:BTreeSet<_>=exact.iter().map(|f|clean(&f.answer)).collect();
                if answers.len()>1 {return Ok(vec![Proposal{fields:[0.0;5],answer:SchoolAnswer::unknown("contradictory_teacher_sources")}]);}
                let first=exact[0];out.push(Proposal{fields:[0.5,0.0,0.0,0.95,0.5],answer:SchoolAnswer{answer:Some(first.answer.clone()),method:"teacher_recall".into(),
                    reason:"retrieved_statement_not_independent_inference".into(),evidence:exact.iter().take(8).map(|f|f.source.clone()).collect(),
                    derivation:None,bound_numbers:vec![],work:self.facts.len() as u64,support:exact.len(),controller_score:None}});
            }
        }
        Ok(out)
    }
    /// Bounded lexical retrieval is explicitly separate from numeric inference.
    /// No nearest-neighbour guess is promoted into a general reasoning rule.
    pub(crate) fn related(&self,question:&str)->Result<Vec<Proposal>> {
        if !text_ok(question){return Err(Error::InvalidData);}
        let key=clean(question);let words:BTreeSet<_>=key.split_whitespace().collect();
        if words.len()<4{return Ok(vec![]);}
        let mut counts:BTreeMap<usize,usize>=BTreeMap::new();
        for word in &words {for &i in self.terms.get(*word).into_iter().flatten(){*counts.entry(i).or_default()+=1;}}
        let mut scored=Vec::new();let query_numbers=ground(question).ok().map(|x|x.1);
        for (i,common) in counts {
            let score=2.0*common as f32/(words.len()+self.word_counts[i]) as f32;
            if score<0.90{continue;}
            let f=&self.facts[i];
            if query_numbers.as_ref().is_some_and(|v|!v.is_empty()) && ground(&f.question).ok().map(|x|x.1)!=query_numbers {continue;}
            scored.push((score,f));
        }
        scored.sort_by(|a,b|b.0.total_cmp(&a.0));
        let Some((score,f))=scored.first().copied() else{return Ok(vec![]);};
        if scored.iter().skip(1).any(|(s,g)|score-*s<0.10&&clean(&f.answer)!=clean(&g.answer)){return Ok(vec![]);}
        Ok(vec![Proposal{fields:[0.0,0.0,0.0,score,0.5],answer:SchoolAnswer{answer:Some(f.answer.clone()),method:"lexical_retrieval".into(),
            reason:"similar_teacher_question_no_semantic_guarantee".into(),evidence:vec![f.source.clone()],derivation:None,bound_numbers:vec![],
            work:self.facts.len() as u64,support:1,controller_score:None}}])
    }

    /// Reuse the acquired expression across related surface forms. A TF-IDF
    /// representation is fitted to TEACHER patterns only; no vocabulary labels
    /// or word->operator dictionary. This is an approximate lexical hypothesis,
    /// kept distinct from exact grounding and never promoted by querying.
    pub(crate) fn lexical_compute(&self,question:&str)->Result<Vec<Proposal>> {
        fn features(s:&str)->BTreeSet<String> {
            let mut out=BTreeSet::new();
            for word in s.split_whitespace().filter(|w|!w.starts_with("slot")) {
                out.insert(format!("word:{word}"));
                let chars:Vec<_>=word.chars().collect();
                for n in [3,4] {for part in chars.windows(n){out.insert(format!("g{n}:{}",part.iter().collect::<String>()));}}
            }out
        }
        fn anchors(s:&str)->Vec<BTreeMap<String,f64>> {
            let words:Vec<_>=s.split_whitespace().collect();let mut out=Vec::new();
            for (i,w) in words.iter().enumerate(){if w.starts_with("slot") {
                let mut a=BTreeMap::new();
                for distance in 1..=2 {if let Some(j)=i.checked_sub(distance){if !words[j].starts_with("slot"){a.insert(words[j].to_string(),3.0/distance as f64);}}}
                if i+1<words.len() && !words[i+1].starts_with("slot"){*a.entry(words[i+1].to_string()).or_default()+=0.5;}
                out.push(a);
            }}out
        }
        let Ok((key,input))=ground(question) else{return Ok(vec![]);};
        if input.is_empty(){return Ok(vec![]);}
        let candidates:Vec<_>=self.skills.iter().filter(|s|s.arity==input.len()).collect();
        if candidates.is_empty(){return Ok(vec![]);}
        let corpus:Vec<_>=self.skills.iter().map(|s|features(&s.pattern)).collect();let mut df:BTreeMap<String,usize>=BTreeMap::new();
        for d in &corpus{for t in d{*df.entry(t.clone()).or_default()+=1;}}
        let weight=|t:&String|1.0+((self.skills.len()+1) as f64/(df.get(t).copied().unwrap_or(0)+1) as f64).ln();
        let query=features(&key);let norm=query.iter().map(|t|weight(t).powi(2)).sum::<f64>().sqrt();
        if norm==0.0{return Ok(vec![]);}
        let mut ranked=Vec::new();let query_anchors=anchors(&key);
        for s in candidates {
            let terms=features(&s.pattern);let other=terms.iter().map(|t|weight(t).powi(2)).sum::<f64>().sqrt();
            let cosine=query.intersection(&terms).map(|t|weight(t).powi(2)).sum::<f64>()/(norm*other).max(1e-12);
            if cosine<0.60 {continue;}
            let reference=anchors(&s.pattern);let mut binding:Vec<_>=(0..input.len()).collect();
            // Exhaustively align numeric slots through their observed local
            // contexts. Reorder only with a uniquely better lexical alignment.
            if input.len()<=3 && query_anchors.len()==input.len() && reference.len()==input.len(){
                let mut permutations=Vec::new();
                fn visit(v:&mut Vec<usize>,used:&mut Vec<bool>,n:usize,out:&mut Vec<Vec<usize>>){if v.len()==n{out.push(v.clone());return;}
                    for i in 0..n{if !used[i]{used[i]=true;v.push(i);visit(v,used,n,out);v.pop();used[i]=false;}}}
                visit(&mut vec![],&mut vec![false;input.len()],input.len(),&mut permutations);
                let alignment=|p:&Vec<usize>|reference.iter().enumerate().map(|(i,a)|a.iter().map(|(word,w)|
                    w*query_anchors[p[i]].get(word).unwrap_or(&0.0)).sum::<f64>()).sum::<f64>();
                permutations.sort_by(|a,b|alignment(b).total_cmp(&alignment(a)));
                if permutations.len()==1 || alignment(&permutations[0])-alignment(&permutations[1])>1.0 {binding=permutations[0].clone();}
            }
            let args:Vec<_>=binding.iter().map(|&i|input[i]).collect();let expr=Expr::Call(s.function,args.iter().copied().map(Expr::integer).collect());
            let mut meter=Meter::new(100_000);
            if let Ok(value)=run(&expr,&[],&self.library,&mut meter){ranked.push((cosine,s,expr,args,value,meter.spent));}
        }
        ranked.sort_by(|a,b|b.0.total_cmp(&a.0));let Some(best)=ranked.first() else{return Ok(vec![]);};
        if ranked.iter().skip(1).any(|other|best.0-other.0<0.10 && best.4!=other.4){return Ok(vec![]);}
        Ok(vec![Proposal{fields:[0.5,0.0,0.0,best.0 as f32,0.4],answer:SchoolAnswer{answer:Some(best.4.to_string()),
            method:"induced_vm_lexical".into(),reason:format!("approximate_lexical_grounding:{:.4}",best.0),
            evidence:vec![best.1.training[0].source.clone(),format!("learned_pattern:{}",best.1.pattern)],derivation:Some(best.2.clone()),bound_numbers:best.3.clone(),
            work:best.5,support:best.1.training.len()+best.1.validation.len(),controller_score:None}}])
    }
    pub fn validate(&self)->Result<()> {
        if let Some(model)=&self.word_problems{model.validate()?;}
        if let Some(model)=&self.quantity_relations{model.validate()?;}
        if self.skills.len()>128 || self.lessons.len()>128 || self.facts.len()>8192{return Err(Error::Capacity);}
        self.library.validate(6)?;
        let mut patterns=BTreeSet::new();let mut meter=Meter::new(20_000_000);
        for s in &self.skills {
            if !patterns.insert(&s.pattern) || !(1..=6).contains(&s.arity) || s.training.len()<18 || s.validation.len()<6
                || s.training.len()+s.validation.len()>128 {return Err(Error::InvalidData);}
            let f=self.library.functions.iter().find(|f|f.id==s.function).ok_or(Error::InvalidProgram)?;
            if f.arity!=s.arity{return Err(Error::Type);}
            for e in s.training.iter().chain(&s.validation) {
                let (key,input)=ground(&e.question)?;
                if key!=s.pattern || input.len()!=s.arity || !text_ok(&e.source){return Err(Error::InvalidData);}
                let expression=Expr::Call(f.id,input.iter().copied().map(Expr::integer).collect());
                if run(&expression,&[],&self.library,&mut meter)?!=e.answer{return Err(Error::InvalidData);}
            }
        }
        if self.facts.iter().any(|f|!text_ok(&f.question)||!text_ok(&f.answer)||!text_ok(&f.source)){return Err(Error::InvalidData);}
        let mut index:BTreeMap<String,Vec<usize>>=BTreeMap::new();let mut terms:BTreeMap<String,Vec<usize>>=BTreeMap::new();let mut lengths=Vec::new();
        for (i,f) in self.facts.iter().enumerate(){let key=clean(&f.question);index.entry(key.clone()).or_default().push(i);
            let words:BTreeSet<_>=key.split_whitespace().map(str::to_string).collect();lengths.push(words.len());for w in words{terms.entry(w).or_default().push(i);}}
        if index!=self.fact_index || terms!=self.terms || lengths!=self.word_counts{return Err(Error::InvalidData);}
        let mut attempted=BTreeSet::new();
        for l in &self.lessons {
            if !attempted.insert(&l.pattern) || l.training<18 || l.validation<6 || l.validation_correct>l.validation
                || (l.expression.is_some() && (l.validation_outcomes.len()!=l.validation || l.validation_outcomes.iter().filter(|&&x|x).count()!=l.validation_correct))
                || (l.accepted && (l.validation_correct!=l.validation || !self.skills.iter().any(|s|s.pattern==l.pattern&&Some(s.function)==l.function))) {return Err(Error::InvalidData);}
        }
        if self.skills.iter().any(|s|!self.lessons.iter().any(|l|l.pattern==s.pattern&&l.accepted)){return Err(Error::InvalidData);}
        Ok(())
    }
}

include!("semantic_grounding.rs");

impl School {
    pub(crate) fn begin_extension(&mut self){self.frozen=false;}
    pub(crate) fn install_word_problems(&mut self,model:crate::word_problems::WordProblemModel)->Result<()> {
        if self.frozen{return Err(Error::Busy);} model.validate()?;self.word_problems=Some(model);Ok(())
    }
    pub fn word_problem_answer(&self,question:&str,prior_only:bool)->Result<crate::word_problems::WordProblemAnswer> {
        // Installed relational learning replaces the default whole-program guess.
        // Keep the previous model for explicit prior-only controls and old states.
        if !prior_only {
            if let Some(model)=&self.quantity_relations {
                let answer=model.answer(question)?;
                return Ok(crate::word_problems::WordProblemAnswer {
                    answer:answer.answer,reason:answer.reason,source_sha256:model.source_sha256.clone(),
                    interpretations:answer.interpretations.into_iter().map(|p|crate::word_problems::Interpretation {
                        program:p.program,equation:p.equation,answer:p.answer,score:p.score,support:0
                    }).collect(),numbers:answer.quantity_values,margin:answer.margin,
                    inference_work:answer.inference_work.saturating_add(answer.candidates_constructed),confidence_is_probability:false,
                });
            }
        }
        self.word_problems.as_ref().ok_or(Error::NoEvidence)?.answer(question,prior_only)
    }
}

impl School {
    pub(crate) fn install_quantity_relations(&mut self, model:crate::quantity_relations::RelationalModel)->Result<()> {
        if self.frozen{return Err(Error::Busy);}model.validate()?;self.quantity_relations=Some(model);Ok(())
    }
    pub fn relational_answer(&self, question:&str)->Result<crate::quantity_relations::RelationalAnswer> {
        self.quantity_relations.as_ref().ok_or(Error::NoEvidence)?.answer(question)
    }
}
