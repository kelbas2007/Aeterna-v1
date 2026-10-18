//! REL-1: learned quantity relevance and ordered relations, then constrained
//! composition. Scores describe hypotheses, never verified natural-language truth.
//! No corpus answer, equation, ID, or program bank is an inference input.
use crate::construction::vm::{Error, Expr, Op, Result};
use crate::word_problems::{execute, ground, GroundedText, Rational};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const MAX_TERMS: usize = 100_000;
const MAX_WORK: u64 = 4_000_000;
const LABELS: [&str; 7] = ["add", "subtract", "reverse_subtract", "multiply", "divide", "reverse_divide", "absent"];
const STOP: &str = "a an the is are was were of at to for and in on by his her their has had have he she it they if then there that this with each how many much what does do did be all will can from as or so now some";

fn slot(s: &str) -> bool { s.strip_prefix("number").is_some_and(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit())) }
fn stem(s: &str) -> String {
    if s.len() > 4 && s.ends_with("ies") { return format!("{}y", &s[..s.len()-3]); }
    if s.len() > 4 && s.ends_with("ing") { return s[..s.len()-3].into(); }
    if s.len() > 3 && s.ends_with('s') && !s.ends_with("ss") { return s[..s.len()-1].into(); }
    s.into()
}
fn punct(s: &str) -> bool { matches!(s, "." | "?" | "!" | ";") }
fn add(out: &mut BTreeMap<String,u32>, s: String) { *out.entry(s).or_default() += 1; }
fn ngrams(out: &mut BTreeMap<String,u32>, prefix: &str, words: &[String], max: usize) {
    for n in 1..=max { for w in words.windows(n) { add(out, format!("{prefix}:{}", w.join("_"))); } }
}
/// Quantity values and absolute variable IDs are deliberately not features.
/// Structural positions, local clauses, question text and lexical overlaps are.
pub fn features(g: &GroundedText, a: usize, b: Option<usize>) -> Result<BTreeMap<String,u32>> {
    let n = g.numbers.len();
    if n == 0 || n > 6 || a >= n || b.is_some_and(|j| j >= n || j == a) || g.tokens.len() > 512 { return Err(Error::InvalidData); }
    let q = g.tokens.iter().rposition(|s| ["how","what","find","calculate","determine"].contains(&s.as_str())).unwrap_or(g.tokens.len());
    let positions: Vec<_> = g.tokens.iter().enumerate().filter(|(_,w)| slot(w)).map(|(i,_)|i).collect();
    if positions.len() != n { return Err(Error::InvalidData); }
    let ti = positions[a]; let tj = b.map(|j| positions[j]);
    let words: Vec<String> = g.tokens.iter().enumerate().map(|(i,w)| {
        if i == ti { "A".into() } else if Some(i) == tj { "B".into() } else if slot(w) { "N".into() } else { stem(w) }
    }).collect();
    let stops: BTreeSet<&str> = STOP.split_whitespace().collect();
    let content = |w: &String| !stops.contains(w.as_str()) && w.bytes().all(|c| c.is_ascii_alphabetic()) && !["A","B","N"].contains(&w.as_str());
    let qa: Vec<_> = words[q..].iter().filter(|w| content(w)).cloned().collect();
    let mut out = BTreeMap::new();
    ngrams(&mut out,"all",&words,2); ngrams(&mut out,"q",&words[q..],3);
    let mut local: Vec<Vec<String>> = Vec::new();
    let mut anchors = vec![("a",ti)]; if let Some(j) = tj { anchors.push(("b",j)); }
    for (p,k) in anchors {
        let mut lo = k; while lo > 0 && !punct(&words[lo-1]) { lo -= 1; }
        let mut hi = k+1; while hi < words.len() && !punct(&words[hi]) { hi += 1; }
        ngrams(&mut out,&format!("{p}s"),&words[lo..hi],2);
        ngrams(&mut out,&format!("{p}l"),&words[lo.max(k.saturating_sub(5))..k],2);
        ngrams(&mut out,&format!("{p}r"),&words[k+1..hi.min(k+6)],2);
        for j in k.saturating_sub(5)..(k+6).min(words.len()) { if j != k { add(&mut out,format!("{p}:{}:{}",j as isize-k as isize,words[j])); } }
        let near: Vec<_> = words[lo.max(k.saturating_sub(5))..hi.min(k+6)].iter().filter(|w| content(w)).cloned().collect();
        add(&mut out,format!("{p}:in_question:{}",usize::from(k >= q)));
        for j in k+1..hi.min(k+5) { if qa.contains(&words[j]) { add(&mut out,format!("{p}:qmatch:{}",j-k)); } }
        for w in &near { for v in &qa { add(&mut out,format!("{p}xq:{w}|{v}")); } }
        local.push(near);
    }
    if let Some(j) = tj {
        add(&mut out,format!("distance:{}",(ti.abs_diff(j)/4).min(5)));
        add(&mut out,format!("order:{}",usize::from(ti < j)));
        add(&mut out,format!("same_clause:{}",usize::from(!words[ti.min(j)+1..ti.max(j)].iter().any(|w| punct(w)))));
        add(&mut out,format!("same_unit:{}",usize::from(ti+1<words.len() && j+1<words.len() && words[ti+1]==words[j+1])));
        for x in &local[0] { for y in &local[1] { add(&mut out,format!("axb:{x}|{y}")); } }
    }
    add(&mut out,format!("count:{n}"));
    if out.len() > 16384 { return Err(Error::Capacity); }
    Ok(out)
}

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactorModel {
    pub vocabulary: Vec<String>, pub idf: Vec<f64>, pub intercept: Vec<f64>,
    pub scales: Vec<f64>,
    #[serde(with="crate::word_problems::matrix_hex")]
    pub weights: Vec<i16>,
}
impl FactorModel {
    fn validate(&self, k: usize) -> Result<()> {
        let f = self.vocabulary.len();
        if f == 0 || f > MAX_TERMS || self.idf.len()!=f || self.intercept.len()!=k || self.scales.len()!=k || self.weights.len()!=f*k
            || self.vocabulary.windows(2).any(|w|w[0]>=w[1]) || self.vocabulary.iter().any(|w|w.is_empty()||w.len()>512||!w.is_ascii())
            || self.idf.iter().any(|v|!v.is_finite()||!(1.0..=32.0).contains(v))
            || self.scales.iter().any(|v|!v.is_finite()||*v<=0.0||*v>1e6) || self.intercept.iter().any(|v|!v.is_finite()) { return Err(Error::InvalidData); }
        Ok(())
    }
    pub fn score(&self, f: &BTreeMap<String,u32>, work: &mut u64) -> Result<Vec<f64>> {
        let mut x = Vec::new(); let mut ss = 0.0;
        for (w,&count) in f {
            if let Ok(i) = self.vocabulary.binary_search(w) {
                if count == 0 { return Err(Error::InvalidData); }
                let v = (1.0+(count as f64).ln())*self.idf[i]; ss += v*v; x.push((i,v));
            }
        }
        let mut out = self.intercept.clone();
        if ss > 0.0 {
            let norm = ss.sqrt(); let k = out.len();
            for (i,v) in x { for (j,s) in out.iter_mut().enumerate() {
                *work = work.checked_add(1).ok_or(Error::Capacity)?;
                if *work > MAX_WORK { return Err(Error::Budget); }
                *s += (v/norm)*self.weights[i*k+j] as f64*self.scales[j];
            } }
        }
        if out.iter().any(|v|!v.is_finite()) { return Err(Error::InvalidData); } Ok(out)
    }
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalModel {
    pub version: u32, pub source_sha256: String, pub supervision: String,
    pub training_examples: usize, pub supported_examples: usize,
    pub relevance_weight: f64, pub max_used: usize, pub beam_per_root: usize,
    pub unary: FactorModel, pub pair: FactorModel,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RelationEvidence { pub first: usize, pub second: usize, pub scores: Vec<f64>, pub selected: String }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RelationalCandidate { pub program: Expr, pub equation: String, pub answer: String, pub score: f64, pub used: Vec<usize> }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RelationalAnswer {
    pub answer: Option<String>, pub reason: String, pub interpretations: Vec<RelationalCandidate>,
    pub quantity_values: Vec<String>, pub unary_scores: Vec<Vec<f64>>, pub relations: Vec<RelationEvidence>,
    pub margin: Option<f64>, pub inference_work: u64, pub candidates_constructed: u64,
    pub confidence_is_probability: bool, pub semantics_verified: bool,
}
#[derive(Clone)]
struct Item { expr: Expr, score: f64, key: String }
fn family(e: &Expr) -> usize { match e { Expr::Bin(Op::Add,_,_)=>0,Expr::Bin(Op::Sub,_,_)=>1,Expr::Bin(Op::Mul,_,_)=>2,Expr::Bin(Op::DivExact,_,_)=>3,_=>4 } }
fn symbol(op: Op) -> &'static str { match op { Op::Add=>"+",Op::Sub=>"-",Op::Mul=>"*",_=>"/" } }
fn tree_key(e: &Expr) -> String { match e { Expr::Input(i)=>format!("n{i}"),Expr::Bin(op,a,b)=>format!("({} {} {})",symbol(*op),tree_key(a),tree_key(b)),_=>"invalid".into() } }
fn fold(mut terms: Vec<Expr>, op: Op) -> Expr {
    terms.sort_by_key(tree_key); let mut it = terms.into_iter(); let mut v = it.next().expect("nonempty by construction");
    for x in it { v = Expr::bin(op,v,x); } v
}
fn collect_same(e: Expr, op: Op, out: &mut Vec<Expr>) {
    match e { Expr::Bin(p,a,b) if p==op=>{collect_same(*a,op,out);collect_same(*b,op,out);},e=>out.push(e) }
}
fn join(op: Op, a: Expr, b: Expr) -> Expr {
    if op==Op::Add || op==Op::Mul { let mut t = Vec::new();collect_same(a,op,&mut t);collect_same(b,op,&mut t);fold(t,op) }
    else { Expr::bin(op,a,b) }
}
fn mask(e: &Expr) -> usize { match e { Expr::Input(i)=>1<<i,Expr::Bin(_,a,b)=>mask(a)|mask(b),_=>0 } }
fn pair_label(e: &Expr, i: usize, j: usize) -> usize {
    let Expr::Bin(op,a,b) = e else { return 6; };
    if mask(e)&(1<<i)==0 || mask(e)&(1<<j)==0 { return 6; }
    let ai = mask(a)&(1<<i)!=0;let aj=mask(a)&(1<<j)!=0;
    if ai && aj { return pair_label(a,i,j); } if !ai && !aj { return pair_label(b,i,j); }
    match op { Op::Add=>0,Op::Sub=>if ai{1}else{2},Op::Mul=>3,Op::DivExact=>if ai{4}else{5},_=>6 }
}
fn printable(e: &Expr, x: &[Rational]) -> String {
    match e { Expr::Input(i)=>x[*i].text(),Expr::Bin(op,a,b)=>format!("({} {} {})",printable(a,x),symbol(*op),printable(b,x)),_=>"invalid".into() }
}
fn keep(items: &mut Vec<Item>, x: Item, beam: usize) {
    if items.iter().any(|p|p.key==x.key) { return; }
    items.push(x); items.sort_by(|a,b|b.score.total_cmp(&a.score).then(a.key.cmp(&b.key))); items.truncate(beam);
}

impl RelationalModel {
    pub fn validate(&self) -> Result<()> {
        if self.version!=1 || self.supervision!="training_derivations" || self.source_sha256.len()!=64 || !self.source_sha256.bytes().all(|b|b.is_ascii_hexdigit())
            || self.training_examples==0 || self.training_examples>100_000 || self.supported_examples==0 || self.supported_examples>self.training_examples
            || !self.relevance_weight.is_finite() || !(0.1..=10.0).contains(&self.relevance_weight) || !(2..=6).contains(&self.max_used) || !(1..=4).contains(&self.beam_per_root) { return Err(Error::InvalidData); }
        self.unary.validate(2)?; self.pair.validate(7)?; Ok(())
    }
    pub fn answer(&self, text: &str) -> Result<RelationalAnswer> {
        let mut out = RelationalAnswer {answer:None,reason:"unsupported_text_or_number_scope".into(),interpretations:vec![],quantity_values:vec![],unary_scores:vec![],relations:vec![],margin:None,inference_work:0,candidates_constructed:0,confidence_is_probability:false,semantics_verified:false};
        let g=match ground(text) { Ok(g)=>g,Err(Error::InvalidData|Error::Capacity)=>return Ok(out),Err(e)=>return Err(e) };
        let n=g.numbers.len(); out.quantity_values=g.numbers.iter().map(Rational::text).collect();
        let mut pairs = vec![vec![vec![0.0;7];n];n];
        for i in 0..n { out.unary_scores.push(self.unary.score(&features(&g,i,None)?,&mut out.inference_work)?);
            for j in i+1..n { pairs[i][j]=self.pair.score(&features(&g,i,Some(j))?,&mut out.inference_work)?; }
        }
        let mut states: Vec<Vec<Vec<Item>>> = vec![vec![Vec::new();5];1<<n];
        let ops=[Op::Add,Op::Sub,Op::Mul,Op::DivExact];
        for set in 1usize..(1<<n) {
            let count=set.count_ones() as usize; if count>self.max_used {continue;}
            if count==1 { let i=set.trailing_zeros() as usize;let e=Expr::Input(i);states[set][4].push(Item{key:tree_key(&e),expr:e,score:0.0});continue; }
            let mut sub=(set-1)&set;
            while sub>0 {
                let rest=set^sub;
                if rest>0 { for (root,&op) in ops.iter().enumerate() {
                    if (op==Op::Add||op==Op::Mul) && sub>rest {continue;}
                    let forbidden=if root<2 {1}else{3};
                    let mut cross=0.0;
                    for i in 0..n { if sub&(1<<i)==0 {continue;} for j in 0..n { if rest&(1<<j)==0 {continue;}
                        let label=match op {Op::Add=>0,Op::Sub=>if i<j{1}else{2},Op::Mul=>3,_=>if i<j{4}else{5}};
                        cross+=pairs[i.min(j)][i.max(j)][label];
                    } }
                    let left=states[sub].clone();let right=states[rest].clone();
                    for (la,aa) in left.iter().enumerate() {if la==forbidden{continue;}for (lb,bb) in right.iter().enumerate(){if lb==forbidden{continue;}
                        for a in aa {for b in bb {
                            out.candidates_constructed+=1;
                            if out.candidates_constructed>250_000 {out.reason="structural_search_budget_exhausted".into();return Ok(out);}
                            let e=join(op,a.expr.clone(),b.expr.clone());
                            let item=Item{key:tree_key(&e),expr:e,score:a.score+b.score+cross};
                            keep(&mut states[set][root],item,self.beam_per_root);
                        }}
                    }}
                }}
                sub=(sub-1)&set;
            }
        }
        let mut final_items=Vec::new();
        for set in 1usize..(1<<n) {
            let mut side=0.0;
            for i in 0..n { side+=self.relevance_weight*out.unary_scores[i][usize::from(set&(1<<i)!=0)];
                for j in i+1..n {if set&(1<<i)==0||set&(1<<j)==0{side+=pairs[i][j][6];}}
            }
            for group in &states[set] {for c in group { if let Ok(value)=execute(&c.expr,&g.numbers) {
                final_items.push((c.score+side,c.key.clone(),c.expr.clone(),value));
            }}}
        }
        final_items.sort_by(|a,b|b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut distinct=BTreeSet::new(); final_items.retain(|x|distinct.insert(x.1.clone()));
        if final_items.is_empty(){out.reason="no_valid_composition".into();return Ok(out);}
        out.margin=final_items.get(1).map(|b|final_items[0].0-b.0);
        out.answer=Some(final_items[0].3.text());out.reason="learned_relational_hypothesis_NOT_semantically_verified".into();
        for i in 0..n {for j in i+1..n {out.relations.push(RelationEvidence{first:i,second:j,scores:pairs[i][j].clone(),selected:LABELS[pair_label(&final_items[0].2,i,j)].into()});}}
        out.interpretations=final_items.into_iter().take(3).map(|(score,_,program,value)| {
            RelationalCandidate{equation:printable(&program,&g.numbers),answer:value.text(),score,used:(0..n).filter(|&i|mask(&program)&(1<<i)!=0).collect(),program}
        }).collect();Ok(out)
    }
}
