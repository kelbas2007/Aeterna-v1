use aeterna_v1::construction::synthesis::*;
use aeterna_v1::construction::vm::*;

fn stream(source:usize,width:usize) -> Vec<Transition> {
    let mut obs=vec![0;width];let mut h=0;let mut rows=Vec::new();
    for i in 0..24 {
        let a=[2,1,0,2,0,1,1][i%7]; let mut post=vec![0;width];
        for j in 0..width {post[j]=((i*17+j*13)%23) as i64-11;}
        post[0]=h*obs[source];h=a as i64;
        rows.push(Transition{id:i as u64,restart:i==0,pre:obs,action:a,post:post.clone()});obs=post;
    }rows
}
#[test] fn every_input_position_can_support_a_new_memory_law() {
    for source in 1..8 {
        let data=stream(source,8);let p=Program::identity(8);let lib=Library::default();
        let cfg=SearchConfig{work:2_000_000,max_expr_nodes:9,max_frontier:32768,max_updates:12,max_state:4,deterministic:false};
        let t=synthesize_requested(&p,&data,&lib,&cfg,&[],Some(0)).unwrap();
        let q=t.winner.unwrap().apply(&p,&lib).unwrap();
        let predictions=replay(&q,&data,&lib,&mut Meter::new(100000)).unwrap().0;
        assert!(predictions.iter().zip(&data).all(|(p,t)|p[0]==t.post[0]),"source={source}, {q:?}");
        assert!(!q.cells.is_empty());assert!(t.work<=cfg.work);
    }
}
#[test] fn learned_root_must_really_be_instantiated_to_receive_credit() {
    let data=stream(6,8);let mut p=Program::identity(8);p.cells.push(StateCell{initial:0,update:Expr::Action});
    let hint=Hint{constructor_id:77,output:Expr::bin(Op::Mul,Expr::Hole{id:0,ty:Ty::Int},Expr::State(0)),cells:p.cells.clone()};
    let cfg=SearchConfig{work:200000,..Default::default()};let lib=Library::default();
    let t=synthesize_requested(&p,&data,&lib,&cfg,&[hint],Some(0)).unwrap();
    assert_eq!(t.constructor_id,Some(77));let q=t.winner.unwrap().apply(&p,&lib).unwrap();
    assert_eq!(q.outputs[0],Expr::bin(Op::Mul,Expr::Input(6),Expr::State(0)));
}
#[test] fn false_one_row_match_is_not_an_exact_completion() {
    let mut data=stream(2,3);data.last_mut().unwrap().post[0]+=1;
    let p=Program::identity(3);let lib=Library::default();
    let t=synthesize_requested(&p,&data,&lib,&SearchConfig{work:20000,..Default::default()},&[],Some(0)).unwrap();
    if let Some(patch)=t.winner {let q=patch.apply(&p,&lib).unwrap();let rows=replay(&q,&data,&lib,&mut Meter::new(100000)).unwrap().0;assert!(rows.iter().zip(&data).any(|(p,t)|p[0]!=t.post[0]));}
}
