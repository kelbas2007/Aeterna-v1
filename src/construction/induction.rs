//! Supervised, bounded expression induction over the existing VM.
//! No subject names, formulas, examination labels or environment IDs are inputs.
//! Observational signatures compress the search, not a proof of equivalence.
use super::vm::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Example { pub input: Vec<i64>, pub output: i64 }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InductionConfig { pub max_nodes: usize, pub max_terms: usize, pub work: u64 }
impl Default for InductionConfig {
    fn default() -> Self { Self { max_nodes: 7, max_terms: 8192, work: 2_000_000 } }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InductionResult { pub expression: Option<Expr>, pub work: u64, pub terms: usize, pub truncated: bool }
#[derive(Clone)]
struct Term { expr: Expr, values: Vec<i64> }
struct Search<'a> {
    rows: &'a [Example], lib: &'a Library, cfg: &'a InductionConfig,
    meter: Meter, layers: Vec<Vec<Term>>, seen: BTreeMap<Vec<i64>, Expr>,
    solution: Option<Expr>, truncated: bool,
}
impl Search<'_> {
    fn insert(&mut self, expr: Expr, values: Vec<i64>) -> Result<bool> {
        self.meter.charge(1 + values.len() as u64)?;
        if expr.nodes() > self.cfg.max_nodes { return Ok(false); }
        if values.iter().zip(self.rows).all(|(&v,r)|v == r.output) {
            // Verify in the actual VM, not only the search's column evaluator.
            for r in self.rows {
                if expr.eval(Context{input:&r.input,action:0,state:&[],args:&[]},self.lib,&mut self.meter)?.int()? != r.output {
                    return Err(Error::InvalidProgram);
                }
            }
            self.solution = Some(expr); return Ok(true);
        }
        if self.seen.contains_key(&values) { return Ok(false); }
        if self.seen.len() >= self.cfg.max_terms { self.truncated=true; return Ok(false); }
        self.layers[expr.nodes()].push(Term{expr:expr.clone(), values:values.clone()});
        self.seen.insert(values,expr); Ok(false)
    }
    fn evaluate(&mut self,e:Expr)->Result<bool> {
        let mut values=Vec::with_capacity(self.rows.len());
        for r in self.rows {
            match e.eval(Context{input:&r.input,action:0,state:&[],args:&[]},self.lib,&mut self.meter) {
                Ok(Value::Int(v))=>values.push(v), Err(Error::Budget)=>return Err(Error::Budget), _=>return Ok(false),
            }
        }
        self.insert(e,values)
    }
    // Generic exact inverse requirements infer constants from training only.
    fn affine(&mut self,term:&Term)->Result<bool> {
        if term.expr.nodes()+2>self.cfg.max_nodes {return Ok(false);}
        for mode in 0..4 {
            self.meter.charge(self.rows.len() as u64)?;
            let required:Option<Vec<i64>>=term.values.iter().zip(self.rows).map(|(&x,r)|match mode {
                0=>r.output.checked_sub(x),
                1=>r.output.checked_add(x),
                2=>if x!=0 && r.output.checked_rem(x)==Some(0) {r.output.checked_div(x)} else {None},
                _=>if r.output!=0 && x.checked_rem(r.output)==Some(0) {x.checked_div(r.output)} else {None},
            }).collect();
            let Some(k)=required.filter(|v| !v.is_empty() && v.iter().all(|x|*x==v[0])).map(|v|v[0]) else{continue;};
            let c=Expr::integer(k);
            let e=match mode {0=>Expr::bin(Op::Add,term.expr.clone(),c),1=>Expr::bin(Op::Sub,c,term.expr.clone()),
                2=>Expr::bin(Op::Mul,term.expr.clone(),c),_=>Expr::bin(Op::DivExact,term.expr.clone(),c)};
            if self.evaluate(e)? {return Ok(true);}
        }
        Ok(false)
    }
    fn run(&mut self)->Result<()> {
        let width=self.rows[0].input.len();
        for c in [0,1,-1,2] {if self.evaluate(Expr::integer(c))? {return Ok(());}}
        for i in 0..width {if self.evaluate(Expr::Input(i))? {return Ok(());}}
        if self.rows.iter().all(|r| r.output==self.rows[0].output) {
            self.evaluate(Expr::integer(self.rows[0].output))?;return Ok(());
        }
        // Previously acquired functions are searched without their names.
        for f in self.lib.functions.clone() {
            if f.arity==0 || f.arity>3 {continue;}
            for code in 0..width.pow(f.arity as u32) {
                let mut n=code;let args=(0..f.arity).map(|_|{let i=n%width;n/=width;Expr::Input(i)}).collect();
                if self.evaluate(Expr::Call(f.id,args))? {return Ok(());}
            }
        }
        for n in 1..=self.cfg.max_nodes {
            if n>=3 {
                for l in 1..n-1 {
                    let r=n-l-1;
                    let left=self.layers[l].clone(); let right=self.layers[r].clone();
                    for a in &left {for b in &right {
                        for op in [Op::Add,Op::Sub,Op::Mul,Op::DivExact,Op::Rem,Op::Xor] {
                            if matches!(op,Op::Add|Op::Mul|Op::Xor) && a.expr>b.expr {continue;}
                            self.meter.charge(self.rows.len() as u64)?;
                            let values:Result<Vec<_>>=a.values.iter().zip(&b.values).map(|(&x,&y)|op.apply(Value::Int(x),Value::Int(y))?.int()).collect();
                            if let Ok(v)=values {if self.insert(Expr::bin(op,a.expr.clone(),b.expr.clone()),v)? {return Ok(());}}
                        }
                    }}
                }
            }
            let layer=self.layers[n].clone();
            for t in &layer {if self.affine(t)? {return Ok(());}}
            // Binary comparisons are generic VM computations, not a rule list.
            if n>=4 {
                for l in 1..n-2 {
                    let r=n-l-2;let left=self.layers[l].clone();let right=self.layers[r].clone();
                    for a in &left {for b in &right {for op in [Op::Eq,Op::Lt] {
                        let e=Expr::AsInt(Box::new(Expr::bin(op,a.expr.clone(),b.expr.clone())));
                        if self.evaluate(e)? {return Ok(());}
                    }}}
                }
            }
        }
        Ok(())
    }
}
pub fn infer(rows:&[Example],lib:&Library,cfg:&InductionConfig)->Result<InductionResult> {
    if rows.len()<4 || rows.len()>128 || !(1..=6).contains(&rows[0].input.len())
        || rows.iter().any(|r|r.input.len()!=rows[0].input.len())
        || !(1..=11).contains(&cfg.max_nodes) || !(16..=32768).contains(&cfg.max_terms)
        || !(100..=20_000_000).contains(&cfg.work) {return Err(Error::InvalidData);}
    lib.validate(rows[0].input.len())?;
    let mut s=Search{rows,lib,cfg,meter:Meter::new(cfg.work),layers:vec![vec![];cfg.max_nodes+1],seen:BTreeMap::new(),solution:None,truncated:false};
    match s.run() {Ok(())=>{},Err(Error::Budget)=>s.truncated=true,Err(e)=>return Err(e)};
    Ok(InductionResult{expression:s.solution,work:s.meter.spent,terms:s.seen.len(),truncated:s.truncated})
}
