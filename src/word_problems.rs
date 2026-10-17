//! External natural-language quantity grounding into acquired arithmetic programs.
//!
//! The lexical ranker is trained from text/numeric-answer examples, not an LLM.
//! Scores rank hypotheses; a margin is NOT a calibrated truth probability.
//! The program language, tokenization and feature families are authored.
use crate::construction::vm::{Error, Expr, Op, Result, Value};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const MAX_INPUT: usize = 4096;
const MAX_TERMS: usize = 40_000;
const MAX_CLASSES: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rational { pub numerator: i128, pub denominator: i128 }
impl Rational {
    fn new(mut a: i128, mut b: i128) -> Result<Self> {
        if b == 0 { return Err(Error::InvalidData); }
        if b < 0 { a = a.checked_neg().ok_or(Error::Overflow)?; b = b.checked_neg().ok_or(Error::Overflow)?; }
        let (mut x, mut y) = (a.checked_abs().ok_or(Error::Overflow)?, b);
        while y != 0 { let r = x % y; x = y; y = r; }
        let d = x.max(1); Ok(Self { numerator: a / d, denominator: b / d })
    }
    pub fn parse(s: &str) -> Result<Self> {
        if s.len() > 48 { return Err(Error::Capacity); }
        if let Some((a,b))=s.split_once('/') {
            return Self::new(a.parse().map_err(|_|Error::InvalidData)?,b.parse().map_err(|_|Error::InvalidData)?);
        }
        let s=s.replace(',', "");
        let negative=s.starts_with('-');let t=s.trim_start_matches(['-','+']);
        let parts:Vec<_>=t.split('.').collect();
        if parts.len()>2 || parts[0].is_empty() || parts.iter().any(|p|p.is_empty()||!p.chars().all(|c|c.is_ascii_digit())) {return Err(Error::InvalidData);}
        let places=parts.get(1).map_or(0,|p|p.len());if places>9{return Err(Error::Capacity);}
        let n=parts.concat().parse::<i128>().map_err(|_|Error::Overflow)?;
        Self::new(if negative {-n}else{n},10i128.pow(places as u32))
    }
    pub fn text(&self)->String {if self.denominator==1{self.numerator.to_string()}else{format!("{}/{}",self.numerator,self.denominator)}}
    fn apply(&self,op:Op,b:&Self)->Result<Self> {
        let mul=|x:i128,y:i128|x.checked_mul(y).ok_or(Error::Overflow);
        match op {
            Op::Add|Op::Sub=> {let a=mul(self.numerator,b.denominator)?;let c=mul(b.numerator,self.denominator)?;
                Self::new(if op==Op::Add{a.checked_add(c)}else{a.checked_sub(c)}.ok_or(Error::Overflow)?,mul(self.denominator,b.denominator)?)},
            Op::Mul=>Self::new(mul(self.numerator,b.numerator)?,mul(self.denominator,b.denominator)?),
            Op::DivExact=>Self::new(mul(self.numerator,b.denominator)?,mul(self.denominator,b.numerator)?),
            _=>Err(Error::InvalidProgram)
        }
    }
}
/// Exact arithmetic on the SAME restricted Expr tree, in a rational domain.
/// Does not change the integer VM's DivExact semantics or grant any authority.
pub fn execute(e:&Expr, input:&[Rational])->Result<Rational> {
    fn run(e:&Expr,x:&[Rational],budget:&mut usize)->Result<Rational> {
        if *budget==0{return Err(Error::Budget);} *budget-=1;
        match e {
            Expr::Input(i)=>x.get(*i).cloned().ok_or(Error::Index),
            Expr::Lit(Value::Int(n))=>Rational::new(*n as i128,1),
            Expr::Bin(op,a,b)=>run(a,x,budget)?.apply(*op,&run(b,x,budget)?),
            _=>Err(Error::InvalidProgram)
        }
    }
    run(e,input,&mut 31)
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct GroundedText { pub tokens:Vec<String>, pub numbers:Vec<Rational> }
/// No corpus ID, expected answer, or equation is accepted by this interface.
pub fn ground(text:&str)->Result<GroundedText> {
    if text.trim().is_empty() || text.len()>MAX_INPUT || text.contains('\0'){return Err(Error::InvalidData);}
    let low=text.to_lowercase();let c:Vec<_>=low.chars().collect();let(mut i,mut tokens,mut numbers)=(0,Vec::new(),Vec::new());
    while i<c.len() {
        let sign=(c[i]=='-'||c[i]=='+')&&i+1<c.len()&&c[i+1].is_ascii_digit()&&(i==0||!c[i-1].is_alphanumeric());
        if c[i].is_ascii_digit()||sign {
            if numbers.len()>=6{return Err(Error::Capacity);}
            let start=i;if sign{i+=1;}
            while i<c.len()&&c[i].is_ascii_digit(){i+=1;}
            // A comma is grouping only when followed by exactly three digits.
            while i+3<c.len()&&c[i]==','&&c[i+1..=i+3].iter().all(|x|x.is_ascii_digit())&&(i+4==c.len()||!c[i+4].is_ascii_digit()) {i+=4;}
            if i+1<c.len()&&c[i]=='.'&&c[i+1].is_ascii_digit(){i+=1;while i<c.len()&&c[i].is_ascii_digit(){i+=1;}}
            let raw:String=c[start..i].iter().collect();numbers.push(Rational::parse(&raw)?);tokens.push(format!("number{}",numbers.len()-1));
        } else if c[i].is_ascii_alphabetic(){
            let start=i;while i<c.len()&&c[i].is_ascii_alphanumeric(){i+=1;}
            let word:String=c[start..i].iter().collect();
            // MP3 is a word, not a quantity. Reserved slots cannot be injected.
            if is_slot(&word){return Err(Error::InvalidData);}
            tokens.push(word);
        } else if c[i].is_alphabetic(){return Err(Error::InvalidData);}
        else {if ".,?!;".contains(c[i]){tokens.push(c[i].to_string());}i+=1;}
    }
    if tokens.len()>512 || numbers.is_empty(){return Err(Error::InvalidData);}
    Ok(GroundedText{tokens,numbers})
}
fn is_slot(s:&str)->bool{s.strip_prefix("number").is_some_and(|t|!t.is_empty()&&t.chars().all(|c|c.is_ascii_digit()))}
pub fn features(g:&GroundedText,kind:&str)->BTreeMap<String,u32> {
    let ts=&g.tokens;let qpos=ts.iter().rposition(|s|["how","what","find","calculate","determine"].contains(&s.as_str())).unwrap_or(ts.len());
    let norm:Vec<_>=ts.iter().map(|s|if is_slot(s){"num".to_string()}else{s.clone()}).collect();
    let parts:Vec<(&str,&[String])>=if kind=="bag"{vec![("all",&norm)]}else{vec![("all",&norm),("q",&norm[qpos..]),("b",&norm[..qpos])]};
    let mut out=BTreeMap::new();let mut add=|s:String|{*out.entry(s).or_insert(0)+=1;};
    for (part,words) in parts {
        for n in 1..=if kind=="bag"{1}else{3} {
            for w in words.windows(n){add(format!("{}:{}",part,w.join("_")));}
        }
    }
    if kind=="role" {
        for (i,s) in ts.iter().enumerate().filter(|(_,s)|is_slot(s)) {
            for j in i.saturating_sub(5)..(i+6).min(ts.len()) {if i!=j{add(format!("{}:{}:{}",s,j as isize-i as isize,norm[j]));}}
            for j in i+1..(i+6).min(ts.len()){if norm[qpos..].contains(&norm[j]){add(format!("{}:qoverlap:{}",s,norm[j]));}}
        }
    }
    add(format!("count:{}",g.numbers.len()));out
}
mod matrix_hex {
    use serde::{Deserialize,Deserializer,Serializer};
    pub fn serialize<S:Serializer>(a:&Vec<i16>,s:S)->Result<S::Ok,S::Error>{
        let mut text=String::with_capacity(a.len()*4);for n in a{use std::fmt::Write;write!(&mut text,"{:04x}",*n as u16).unwrap();}s.serialize_str(&text)
    }
    pub fn deserialize<'de,D:Deserializer<'de>>(d:D)->Result<Vec<i16>,D::Error>{
        let text=String::deserialize(d)?;
        if text.len()%4!=0||text.len()>super::MAX_TERMS*super::MAX_CLASSES*4{return Err(serde::de::Error::custom("matrix capacity"));}
        text.as_bytes().chunks_exact(4).map(|b|u16::from_str_radix(std::str::from_utf8(b).map_err(serde::de::Error::custom)?,16).map(|v|v as i16).map_err(serde::de::Error::custom)).collect()
    }
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcquiredProgram {pub expression:Expr,pub support:usize}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WordProblemModel {
    pub version:u32,pub feature_kind:String,pub source_sha256:String,pub training_examples:usize,
    pub supported_training_examples:usize,pub vocabulary:Vec<String>,pub idf:Vec<f64>,
    pub programs:Vec<AcquiredProgram>,pub intercept:Vec<f64>,pub scales:Vec<f64>,
    #[serde(with="matrix_hex")] pub weights:Vec<i16>,
    pub developer_margin:f64,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Interpretation {pub program:Expr,pub equation:String,pub answer:String,pub score:f64,pub support:usize}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct WordProblemAnswer {
    pub answer:Option<String>,pub reason:String,pub source_sha256:String,pub interpretations:Vec<Interpretation>,
    pub numbers:Vec<String>,pub margin:Option<f64>,pub inference_work:u64,
    pub confidence_is_probability:bool,
}
fn signature(e:&Expr,depth:usize)->Result<(usize,usize)> {
    if depth>4{return Err(Error::InvalidProgram);}
    match e {
        Expr::Input(i) if *i<6=>Ok((1,1<<i)),
        Expr::Bin(Op::Add|Op::Sub|Op::Mul|Op::DivExact,a,b)=>{let (n,x)=signature(a,depth+1)?;let(m,y)=signature(b,depth+1)?;
            if x&y!=0 || n+m+1>5{return Err(Error::InvalidProgram);}Ok((n+m+1,x|y))},
        _=>Err(Error::InvalidProgram)
    }
}
fn equation(e:&Expr,values:&[Rational])->String {
    match e {
        Expr::Input(i)=>values[*i].text(),
        Expr::Bin(op,a,b)=>format!("({} {} {})",equation(a,values),match op{Op::Add=>"+",Op::Sub=>"-",Op::Mul=>"*",_=>"/"},equation(b,values)),
        _=>"unsupported".into()
    }
}
impl WordProblemModel {
    pub fn validate(&self)->Result<()> {
        let f=self.vocabulary.len();let k=self.programs.len();
        if self.version!=1 || !["role","bag"].contains(&self.feature_kind.as_str()) || self.source_sha256.len()!=64 || !self.source_sha256.chars().all(|c|c.is_ascii_hexdigit())
            || f==0||f>MAX_TERMS||k<2||k>MAX_CLASSES||self.idf.len()!=f||self.intercept.len()!=k||self.scales.len()!=k||self.weights.len()!=f*k
            || self.training_examples==0 || self.training_examples>100_000 || self.supported_training_examples>self.training_examples
            || self.vocabulary.windows(2).any(|w|w[0]>=w[1])||self.vocabulary.iter().any(|w|w.is_empty()||w.len()>512)
            || self.idf.iter().any(|x|!x.is_finite()||*x<1.0||*x>30.0)||self.intercept.iter().any(|x|!x.is_finite())
            || self.scales.iter().any(|x|!x.is_finite()||*x<=0.0||*x>1e6)||!self.developer_margin.is_finite()||self.developer_margin<0.0{return Err(Error::InvalidData);}
        let mut seen=std::collections::BTreeSet::new();let mut support=0usize;
        for p in &self.programs {
            signature(&p.expression,0)?;
            if p.support==0||p.support>self.supported_training_examples||!seen.insert(&p.expression){return Err(Error::InvalidData);}support+=p.support;
        }
        if support!=self.supported_training_examples{return Err(Error::InvalidData);}Ok(())
    }
    pub fn answer(&self,text:&str,prior_only:bool)->Result<WordProblemAnswer> {
        let mut out=WordProblemAnswer{answer:None,reason:"unsupported_text_or_number_scope".into(),source_sha256:self.source_sha256.clone(),interpretations:vec![],numbers:vec![],margin:None,inference_work:0,confidence_is_probability:false};
        let g=match ground(text){Ok(g)=>g,Err(Error::InvalidData|Error::Capacity)=>return Ok(out),Err(e)=>return Err(e)};
        out.numbers=g.numbers.iter().map(Rational::text).collect();
        let mut x=Vec::new();let mut sum=0.0;
        if !prior_only {for (word,count) in features(&g,&self.feature_kind){
            if let Ok(i)=self.vocabulary.binary_search(&word){let v=(1.0+(count as f64).ln())*self.idf[i];sum+=v*v;x.push((i,v));}
        }}
        let mut scores=self.intercept.clone();
        if prior_only {for (s,p) in scores.iter_mut().zip(&self.programs){*s=(p.support as f64).ln();}}
        else if sum>0.0 {let norm=sum.sqrt();let k=scores.len();
            for (i,v) in x{for j in 0..k{scores[j]+=(v/norm)*self.weights[i*k+j] as f64*self.scales[j];out.inference_work+=1;}}
        }
        let mut ranked=Vec::new();
        for (i,p) in self.programs.iter().enumerate(){if let Ok(value)=execute(&p.expression,&g.numbers){ranked.push((i,value));}}
        ranked.sort_by(|(i,_),(j,_)|scores[*j].total_cmp(&scores[*i]).then(i.cmp(j)));
        if ranked.is_empty(){out.reason="no_executable_acquired_program".into();return Ok(out);}
        let first=ranked[0].0;let margin=ranked.get(1).map(|(i,_)|scores[first]-scores[*i]);
        out.margin=margin;out.answer=Some(ranked[0].1.text());
        out.reason=if margin.is_some_and(|m|m>=self.developer_margin){"learned_interpretation_high_margin_NOT_verified"}else{"learned_interpretation_uncertain"}.into();
        out.interpretations=ranked.into_iter().take(3).map(|(i,n)|Interpretation{program:self.programs[i].expression.clone(),equation:equation(&self.programs[i].expression,&g.numbers),answer:n.text(),score:scores[i],support:self.programs[i].support}).collect();Ok(out)
    }
}
