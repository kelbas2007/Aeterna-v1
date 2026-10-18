//! Bounded exact completion of the ACTUAL search root, not a template for a
//! particular world. One or two typed holes get leaf bindings; a single integer
//! hole also gets every existing arithmetic primitive over two leaves. The
//! ordinary grammar remains the fallback. A training match is never promotion.
use super::synthesis::Transition;
use super::vm::*;
use std::collections::BTreeMap;

pub(super) fn complete(root: &Expr,data: &[Transition],states: &[Vec<i64>],target: usize,width: usize,max_nodes: usize,lib: &Library,meter: &mut Meter) -> Result<Option<Expr>> {
    if data.is_empty() || data.len()!=states.len() || target>=width { return Err(Error::InvalidData); }
    let mut holes=BTreeMap::new();
    meter.charge(root.nodes() as u64)?;
    root.map(&mut |e| { if let Expr::Hole{id,ty}=e { holes.insert(*id,*ty); } None });
    if holes.len()>2 { return Ok(None); }
    let holes:Vec<_>=holes.into_iter().collect();
    let mut witness=0;
    for (i,t) in data.iter().enumerate() {
        meter.charge(1)?;
        if (t.post[target] as i128).abs() > (data[witness].post[target] as i128).abs() { witness=i; }
    }
    // A past high-contrast witness avoids spending a full replay on expressions
    // already refuted. ALL remaining rows are required before returning a fit.
    let accepts = |e: &Expr,m: &mut Meter| -> Result<bool> {
        m.charge(e.nodes() as u64)?;
        if e.nodes()>max_nodes || e.ty(lib)!=Ok(Ty::Int) { return Ok(false); }
        for k in 0..data.len() {
            let i=(witness+k)%data.len(); let t=&data[i];
            match e.eval(Context{input:&t.pre,action:t.action,state:&states[i],args:&[]},lib,m) {
                Ok(Value::Int(v)) if v==t.post[target]=>{},
                Err(Error::Budget)=>return Err(Error::Budget),
                _=>return Ok(false),
            }
        }
        Ok(true)
    };
    if holes.is_empty() { return Ok(if accepts(root,meter)? {Some(root.clone())} else {None}); }
    let leaves = |ty| -> Vec<Expr> {
        if ty==Ty::Bool { return vec![Expr::Lit(Value::Bool(false)),Expr::Lit(Value::Bool(true))]; }
        [0,1,-1,2].into_iter().map(Expr::integer).chain([Expr::Action])
            .chain((0..width).map(Expr::Input)).chain((0..states[0].len()).map(Expr::State)).collect()
    };
    let first=leaves(holes[0].1);
    for a in &first {
        meter.charge(root.nodes() as u64)?;
        let partial=root.substitute(holes[0].0,a);
        if holes.len()==1 {
            if accepts(&partial,meter)? {return Ok(Some(partial));}
        } else {
            for b in leaves(holes[1].1) {
                meter.charge(partial.nodes() as u64)?;
                let e=partial.substitute(holes[1].0,&b);
                if accepts(&e,meter)? {return Ok(Some(e));}
            }
        }
    }
    if holes.len()==1 && holes[0].1==Ty::Int {
        for op in [Op::Add,Op::Sub,Op::Mul,Op::Xor,Op::Rem] {
            for a in &first { for b in &first {
                meter.charge((root.nodes()+3) as u64)?;
                let e=root.substitute(holes[0].0,&Expr::bin(op,a.clone(),b.clone()));
                if accepts(&e,meter)? {return Ok(Some(e));}
            }}
        }
    }
    Ok(None)
}
