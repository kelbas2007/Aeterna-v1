//! Metered, side-effect-free instruction language for directed construction.
//! Programs contain no environment identifiers, authority handles or evaluator.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
    Budget, Capacity, Type, Index, Overflow, Hole, InvalidProgram,
    InvalidData, Stale, Busy, NoEvidence, Execution(String), Blocked,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meter { pub limit: u64, pub spent: u64 }
impl Meter {
    pub fn new(limit: u64) -> Self { Self { limit, spent: 0 } }
    pub fn charge(&mut self, n: u64) -> Result<()> {
        let next = self.spent.checked_add(n).ok_or(Error::Budget)?;
        if next > self.limit { self.spent = self.limit; return Err(Error::Budget); }
        self.spent = next; Ok(())
    }
    pub fn remaining(&self) -> u64 { self.limit.saturating_sub(self.spent) }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Ty { Int, Bool }
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Value { Int(i64), Bool(bool) }
impl Value {
    pub fn ty(self) -> Ty { match self { Self::Int(_) => Ty::Int, Self::Bool(_) => Ty::Bool } }
    pub fn int(self) -> Result<i64> { if let Self::Int(x) = self { Ok(x) } else { Err(Error::Type) } }
    pub fn boolean(self) -> Result<bool> { if let Self::Bool(x) = self { Ok(x) } else { Err(Error::Type) } }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Op { Add, Sub, Mul, Xor, Rem, Eq, Lt, And, Or }
impl Op {
    pub fn input(self) -> Ty { if matches!(self, Self::And | Self::Or) { Ty::Bool } else { Ty::Int } }
    pub fn output(self) -> Ty { if matches!(self, Self::Eq | Self::Lt | Self::And | Self::Or) { Ty::Bool } else { Ty::Int } }
    pub fn apply(self, a: Value, b: Value) -> Result<Value> {
        use Value::*;
        Ok(match self {
            Self::And => Bool(a.boolean()? && b.boolean()?),
            Self::Or => Bool(a.boolean()? || b.boolean()?),
            Self::Eq => Bool(a.int()? == b.int()?),
            Self::Lt => Bool(a.int()? < b.int()?),
            Self::Add => Int(a.int()?.checked_add(b.int()?).ok_or(Error::Overflow)?),
            Self::Sub => Int(a.int()?.checked_sub(b.int()?).ok_or(Error::Overflow)?),
            Self::Mul => Int(a.int()?.checked_mul(b.int()?).ok_or(Error::Overflow)?),
            Self::Xor => Int(a.int()? ^ b.int()?),
            Self::Rem => Int(a.int()?.checked_rem(b.int()?).ok_or(Error::Overflow)?),
        })
    }
    /// Sound inverse for one integer hole. None means do not prune.
    pub fn right_requirement(self, left: i64, result: i64) -> Option<i64> {
        match self {
            Self::Add => result.checked_sub(left),
            Self::Sub => left.checked_sub(result),
            Self::Xor => Some(left ^ result),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Expr {
    Lit(Value), Input(usize), Action, State(usize), Arg(usize),
    Hole { id: u16, ty: Ty },
    Bin(Op, Box<Expr>, Box<Expr>),
    Not(Box<Expr>), AsInt(Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Call(u64, Vec<Expr>),
    /// Repeated application of body; Arg(0)=accumulator, Arg(1)=iteration.
    Fold { count: Box<Expr>, seed: Box<Expr>, body: Box<Expr> },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Function { pub id: u64, pub arity: usize, pub body: Expr }
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Library { pub functions: Vec<Function> }
#[derive(Debug, Clone, Copy)]
pub struct Context<'a> { pub input: &'a [i64], pub action: usize, pub state: &'a [i64], pub args: &'a [i64] }

impl Expr {
    pub fn integer(x: i64) -> Self { Self::Lit(Value::Int(x)) }
    pub fn bin(op: Op, a: Self, b: Self) -> Self { Self::Bin(op, Box::new(a), Box::new(b)) }
    pub fn nodes(&self) -> usize {
        match self {
            Self::Bin(_, a, b) => 1 + a.nodes() + b.nodes(),
            Self::Not(a) | Self::AsInt(a) => 1 + a.nodes(),
            Self::If(a,b,c) => 1 + a.nodes() + b.nodes() + c.nodes(),
            Self::Fold { count, seed, body } => 1 + count.nodes() + seed.nodes() + body.nodes(),
            Self::Call(_, args) => 1 + args.iter().map(Self::nodes).sum::<usize>(),
            _ => 1,
        }
    }
    pub fn ty(&self, library: &Library) -> Result<Ty> { self.check_type(library, 0) }
    fn check_type(&self, lib: &Library, depth: usize) -> Result<Ty> {
        if depth > 48 { return Err(Error::Capacity); }
        let sub = |e: &Expr| e.check_type(lib, depth + 1);
        match self {
            Self::Lit(v) => Ok(v.ty()),
            Self::Hole { ty, .. } => Ok(*ty),
            Self::Input(_) | Self::Action | Self::State(_) | Self::Arg(_) => Ok(Ty::Int),
            Self::Bin(op,a,b) => if sub(a)? == op.input() && sub(b)? == op.input() { Ok(op.output()) } else { Err(Error::Type) },
            Self::Not(a) => if sub(a)? == Ty::Bool { Ok(Ty::Bool) } else { Err(Error::Type) },
            Self::AsInt(a) => if sub(a)? == Ty::Bool { Ok(Ty::Int) } else { Err(Error::Type) },
            Self::If(c,a,b) => if sub(c)? == Ty::Bool && sub(a)? == sub(b)? { sub(a) } else { Err(Error::Type) },
            Self::Call(id,args) => {
                let f = lib.functions.iter().find(|f| f.id == *id).ok_or(Error::InvalidProgram)?;
                if f.arity != args.len() || args.iter().any(|a| sub(a) != Ok(Ty::Int)) { return Err(Error::Type); }
                sub(&f.body)
            }
            Self::Fold { count, seed, body } => {
                if sub(count)? == Ty::Int && sub(seed)? == Ty::Int && sub(body)? == Ty::Int { Ok(Ty::Int) } else { Err(Error::Type) }
            }
        }
    }
    pub fn eval(&self, ctx: Context<'_>, library: &Library, meter: &mut Meter) -> Result<Value> {
        self.run(ctx, library, meter, 0)
    }
    fn run(&self, ctx: Context<'_>, lib: &Library, meter: &mut Meter, depth: usize) -> Result<Value> {
        meter.charge(1)?;
        if depth > 48 { return Err(Error::Capacity); }
        let next = depth + 1;
        match self {
            Self::Lit(v) => Ok(*v),
            Self::Input(i) => ctx.input.get(*i).copied().map(Value::Int).ok_or(Error::Index),
            Self::Action => Ok(Value::Int(i64::try_from(ctx.action).map_err(|_| Error::Overflow)?)),
            Self::State(i) => ctx.state.get(*i).copied().map(Value::Int).ok_or(Error::Index),
            Self::Arg(i) => ctx.args.get(*i).copied().map(Value::Int).ok_or(Error::Index),
            Self::Hole { .. } => Err(Error::Hole),
            Self::Bin(op,a,b) => op.apply(a.run(ctx,lib,meter,next)?, b.run(ctx,lib,meter,next)?),
            Self::Not(a) => Ok(Value::Bool(!a.run(ctx,lib,meter,next)?.boolean()?)),
            Self::AsInt(a) => Ok(Value::Int(i64::from(a.run(ctx,lib,meter,next)?.boolean()?))),
            Self::If(c,a,b) => if c.run(ctx,lib,meter,next)?.boolean()? { a.run(ctx,lib,meter,next) } else { b.run(ctx,lib,meter,next) },
            Self::Call(id,args) => {
                let f = lib.functions.iter().find(|f| f.id == *id).ok_or(Error::InvalidProgram)?;
                if args.len() != f.arity { return Err(Error::Type); }
                let values = args.iter().map(|a| a.run(ctx,lib,meter,next)?.int()).collect::<Result<Vec<_>>>()?;
                f.body.run(Context { args: &values, ..ctx }, lib, meter, next)
            }
            Self::Fold { count, seed, body } => {
                let n = count.run(ctx,lib,meter,next)?.int()?;
                if !(0..=4096).contains(&n) { return Err(Error::Capacity); }
                let mut acc = seed.run(ctx,lib,meter,next)?.int()?;
                for i in 0..n {
                    acc = body.run(Context { args: &[acc,i], ..ctx },lib,meter,next)?.int()?;
                }
                Ok(Value::Int(acc))
            }
        }
    }
    pub fn first_hole(&self) -> Option<(u16,Ty)> {
        match self {
            Self::Hole { id, ty } => Some((*id,*ty)),
            Self::Bin(_,a,b) => a.first_hole().or_else(|| b.first_hole()),
            Self::Not(a) | Self::AsInt(a) => a.first_hole(),
            Self::If(c,a,b) => c.first_hole().or_else(|| a.first_hole()).or_else(|| b.first_hole()),
            Self::Call(_,args) => args.iter().find_map(Self::first_hole),
            Self::Fold { count, seed, body } => count.first_hole().or_else(|| seed.first_hole()).or_else(|| body.first_hole()),
            _ => None,
        }
    }
    /// All occurrences of a hole are replaced together (binding consistency).
    pub fn substitute(&self, id: u16, value: &Self) -> Self {
        self.map(&mut |e| if matches!(e, Self::Hole { id: h, .. } if *h == id) { Some(value.clone()) } else { None })
    }
    pub fn map(&self, f: &mut impl FnMut(&Expr) -> Option<Expr>) -> Expr {
        if let Some(e) = f(self) { return e; }
        match self {
            Self::Bin(op,a,b) => Self::bin(*op,a.map(f),b.map(f)),
            Self::Not(a) => Self::Not(Box::new(a.map(f))),
            Self::AsInt(a) => Self::AsInt(Box::new(a.map(f))),
            Self::If(c,a,b) => Self::If(Box::new(c.map(f)),Box::new(a.map(f)),Box::new(b.map(f))),
            Self::Call(id,args) => Self::Call(*id,args.iter().map(|a| a.map(f)).collect()),
            Self::Fold { count, seed, body } => Self::Fold { count: Box::new(count.map(f)), seed: Box::new(seed.map(f)), body: Box::new(body.map(f)) },
            e => e.clone(),
        }
    }
    pub fn references_valid(&self, width: usize, states: usize, args: usize, lib: &Library) -> bool {
        let mut valid = true;
        self.map(&mut |e| {
            match e {
                Self::Input(i) if *i >= width => valid = false,
                Self::State(i) if *i >= states => valid = false,
                Self::Arg(i) if *i >= args => valid = false,
                Self::Hole { .. } => valid = false,
                Self::Fold { count, seed, body } => {
                    valid &= count.references_valid(width,states,args,lib)
                        && seed.references_valid(width,states,args,lib)
                        && body.references_valid(width,states,2,lib);
                    return Some(e.clone());
                }
                Self::Call(id,_) if !lib.functions.iter().any(|f| f.id == *id) => valid = false,
                _ => (),
            }
            None
        });
        valid
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateCell { pub initial: i64, pub update: Expr }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program { pub outputs: Vec<Expr>, pub cells: Vec<StateCell> }
impl Program {
    pub fn identity(width: usize) -> Self { Self { outputs: (0..width).map(Expr::Input).collect(), cells: vec![] } }
    pub fn nodes(&self) -> usize {
        self.outputs.iter().map(Expr::nodes).sum::<usize>() + self.cells.iter().map(|c| 1+c.update.nodes()).sum::<usize>()
    }
    pub fn fingerprint(&self) -> u64 {
        // Stable diagnostic identity, NOT a cryptographic authenticity claim.
        fingerprint(&serde_json::to_vec(self).expect("serializable program"))
    }
    pub fn validate(&self, width: usize, lib: &Library) -> Result<()> {
        if width == 0 || width > 32 || self.outputs.len() != width || self.cells.len() > 8 { return Err(Error::Capacity); }
        // Check bounded depth/types before recursive size traversal.
        for e in self.outputs.iter().chain(self.cells.iter().map(|c| &c.update)) {
            if e.ty(lib)? != Ty::Int { return Err(Error::Type); }
            if !e.references_valid(width,self.cells.len(),0,lib) { return Err(Error::InvalidProgram); }
        }
        if self.nodes() > 512 { return Err(Error::Capacity); }
        Ok(())
    }
    pub fn initial_state(&self) -> Vec<i64> { self.cells.iter().map(|c| c.initial).collect() }
    /// Outputs and all updates read the SAME old state; no outcome is an input.
    pub fn step(&self, input: &[i64], action: usize, state: &[i64], lib: &Library, meter: &mut Meter) -> Result<(Vec<i64>,Vec<i64>)> {
        if self.outputs.len() != input.len() || self.cells.len() != state.len() { return Err(Error::InvalidData); }
        let ctx = Context { input, action, state, args: &[] };
        let outputs = self.outputs.iter().map(|e| e.eval(ctx,lib,meter)?.int()).collect::<Result<Vec<_>>>()?;
        let next = self.cells.iter().map(|c| c.update.eval(ctx,lib,meter)?.int()).collect::<Result<Vec<_>>>()?;
        Ok((outputs,next))
    }
}
impl Library {
    pub fn nodes(&self) -> usize { self.functions.iter().map(|f| 1+f.body.nodes()).sum() }
    pub fn validate(&self, width: usize) -> Result<()> {
        if self.functions.len() > 32 { return Err(Error::Capacity); }
        let mut prefix = Library::default();
        for f in &self.functions {
            if f.arity > 8 || prefix.functions.iter().any(|p| p.id == f.id) { return Err(Error::InvalidProgram); }
            if f.body.ty(&prefix)? != Ty::Int || !f.body.references_valid(width,0,f.arity,&prefix) { return Err(Error::InvalidProgram); }
            prefix.functions.push(f.clone());
        }
        if self.nodes() > 2048 { return Err(Error::Capacity); }
        Ok(())
    }
}
pub fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |h,b| (h ^ u64::from(*b)).wrapping_mul(0x100000001b3))
}

/// Anti-unification preserves identical subtrees and repeated variable binding.
/// It is syntactic generalization, NOT proof of empirical equivalence.
pub fn generalize(a: &Expr, b: &Expr, lib: &Library) -> Result<Expr> {
    fn walk(a: &Expr,b: &Expr,lib: &Library,vars: &mut BTreeMap<(Expr,Expr),u16>) -> Result<Expr> {
        if a == b { return Ok(a.clone()); }
        if a.ty(lib)? != b.ty(lib)? { return Err(Error::Type); }
        Ok(match (a,b) {
            (Expr::Bin(op,a1,a2),Expr::Bin(oq,b1,b2)) if op == oq => Expr::bin(*op,walk(a1,b1,lib,vars)?,walk(a2,b2,lib,vars)?),
            (Expr::Not(a),Expr::Not(b)) => Expr::Not(Box::new(walk(a,b,lib,vars)?)),
            (Expr::AsInt(a),Expr::AsInt(b)) => Expr::AsInt(Box::new(walk(a,b,lib,vars)?)),
            (Expr::If(c,a,b),Expr::If(d,x,y)) => Expr::If(Box::new(walk(c,d,lib,vars)?),Box::new(walk(a,x,lib,vars)?),Box::new(walk(b,y,lib,vars)?)),
            _ => {
                let next = u16::try_from(vars.len()).map_err(|_| Error::Capacity)?;
                let id = *vars.entry((a.clone(),b.clone())).or_insert(next);
                Expr::Hole { id, ty: a.ty(lib)? }
            }
        })
    }
    walk(a,b,lib,&mut BTreeMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn types_and_checked_arithmetic() {
        let lib = Library::default(); let mut meter = Meter::new(100);
        let ctx = Context { input: &[],action: 0,state: &[],args: &[] };
        assert_eq!(Expr::bin(Op::Add,Expr::integer(i64::MAX),Expr::integer(1)).eval(ctx,&lib,&mut meter),Err(Error::Overflow));
        assert_eq!(Expr::bin(Op::Add,Expr::Lit(Value::Bool(true)),Expr::integer(1)).ty(&lib),Err(Error::Type));
        assert_eq!(Op::Add.right_requirement(7,10),Some(3));
        assert_eq!(Op::Mul.right_requirement(0,0),None);
    }
    #[test] fn bounded_loop_and_fuel() {
        let e = Expr::Fold { count: Box::new(Expr::integer(5)),seed: Box::new(Expr::integer(0)),body: Box::new(Expr::bin(Op::Add,Expr::Arg(0),Expr::integer(2))) };
        let ctx = Context { input: &[],action: 0,state: &[],args: &[] };
        assert_eq!(e.eval(ctx,&Library::default(),&mut Meter::new(100)),Ok(Value::Int(10)));
        assert_eq!(e.eval(ctx,&Library::default(),&mut Meter::new(3)),Err(Error::Budget));
    }
    #[test] fn simultaneous_state_and_no_post_leak() {
        let p = Program { outputs: vec![Expr::State(0)],cells: vec![StateCell { initial: 0,update: Expr::Action },StateCell { initial: 9,update: Expr::State(0) }] };
        p.validate(1,&Library::default()).unwrap();
        assert_eq!(p.step(&[88],1,&[0,9],&Library::default(),&mut Meter::new(20)).unwrap(),(vec![0],vec![1,0]));
    }
    #[test] fn anti_unification_reuses_variable() {
        let a = Expr::bin(Op::Add,Expr::Input(0),Expr::Input(0));
        let b = Expr::bin(Op::Add,Expr::Input(1),Expr::Input(1));
        let g = generalize(&a,&b,&Library::default()).unwrap();
        assert_eq!(g.substitute(0,&Expr::Input(2)),Expr::bin(Op::Add,Expr::Input(2),Expr::Input(2)));
    }
    #[test] fn library_rejects_recursive_call_and_bad_state() {
        let lib = Library { functions: vec![Function { id: 1,arity: 0,body: Expr::Call(1,vec![]) }] };
        assert!(lib.validate(1).is_err());
        assert!(Program { outputs: vec![Expr::State(9)],cells: vec![] }.validate(1,&Library::default()).is_err());
    }
}
