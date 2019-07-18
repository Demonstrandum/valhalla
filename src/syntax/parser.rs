use super::token;
use super::ast;
use super::operators;

use token::{Token, TokenType};
use ast::Nodes;

pub fn parse(stream : Vec<Token>) -> ast::Root {
    let mut environment = ParseEnvironment::new(stream);
    environment.optable.new_fun("max", 4);

    environment.start();

    environment.root
}

struct ParseEnvironment {
    pub root : ast::Root,
    pub stream : Vec<Token>,
    pub optable : operators::PrecedenceTable
}

impl ParseEnvironment {
    pub fn new(stream : Vec<Token>) -> Self {
        ParseEnvironment {
            root: ast::Root::new(),
            stream: stream,
            optable: operators::PrecedenceTable::new()
        }
    }
    
    pub fn start(&mut self) {
        let mut current = self.stream.first();
        while current.is_some() && current.unwrap().class != TokenType::EOF {
            if current.unwrap().class == TokenType::Term {
                self.stream.remove(0);
                current = self.stream.get(0);
                continue;
            }
            let e = self.expr(0);
            self.root.branches.push(e);
            current = self.stream.get(0);
        }
    }

    fn null_den(&mut self, token : &Token) -> Nodes {
        match token.class {
            TokenType::Ident => ast::IdentNode::new(&token.string),
            TokenType::Op => {  // Prefix Op.
                let op = self.optable.lookup(&token.string, 1);
                if op.is_some() {
                    return ast::CallNode::new(ast::IdentNode::new(&token.string), vec![self.expr(300)]);
                }
                return panic!("`{}` is not a prefix operator.", token.string);
            },
            TokenType::Num => ast::NumNode::new(&*token.string),
            TokenType::Str => ast::StrNode::new(&token.string),
            _ => panic!("Passed non-atomic token to `atom` parser.")
        }
    }

    fn expr(&mut self, right_prec : i32) -> Nodes {
        let popped = &self.stream.remove(0);
        let mut left = self.null_den(popped);

        while self.optable.precedence(&self.stream[0].string).unwrap_or(0) > right_prec {
            if self.stream[0].class == TokenType::EOF { break; }
            let op = self.optable.lookup(&self.stream.remove(0).string, 2).unwrap();
            left = self.left_den(left, op.clone());
        }
        return left;
    }

    fn left_den(&mut self, left : Nodes, op : operators::Operator) -> Nodes {
        let right = self.expr(op.precedence - (if op.is_right() { 1 } else { 0 }));
        ast::CallNode::new(ast::IdentNode::new(op.name), vec![left, right])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn numeric_parsing() {
        assert_eq!(ast::NumNode::new(2).num().unwrap().value, Numerics::Natural(2usize));
        assert_eq!(ast::NumNode::new(2usize).num().unwrap().value, Numerics::Natural(2usize));
        assert_eq!(ast::NumNode::new(2u32).num().unwrap().value, Numerics::Natural(2usize));
        assert_eq!(ast::NumNode::new(2i32).num().unwrap().value, Numerics::Natural(2usize));

        assert_eq!(ast::NumNode::new(-2).num().unwrap().value, Numerics::Integer(-2isize));
        assert_eq!(ast::NumNode::new(-2i32).num().unwrap().value, Numerics::Integer(-2isize));
        assert_eq!(ast::NumNode::new(-2isize).num().unwrap().value, Numerics::Integer(-2isize));
        
        assert_eq!(ast::NumNode::new(-2.62).num().unwrap().value, Numerics::Real(-2.62f64));
        assert_eq!(ast::NumNode::new(2.62).num().unwrap().value, Numerics::Real(2.62f64));

        assert_eq!(ast::NumNode::new("2").num().unwrap().value, Numerics::Natural(2));
        assert_eq!(ast::NumNode::new("325").num().unwrap().value, Numerics::Natural(325));
        assert_eq!(ast::NumNode::new("0b01010110").num().unwrap().value, Numerics::Natural(0b01010110));
        assert_eq!(ast::NumNode::new("0o721").num().unwrap().value, Numerics::Natural(0o721));
        assert_eq!(ast::NumNode::new("0xfa").num().unwrap().value, Numerics::Natural(0xfa));
        assert_eq!(ast::NumNode::new("0xf").num().unwrap().value, Numerics::Natural(0xf));
        assert_eq!(ast::NumNode::new("2.672").num().unwrap().value, Numerics::Real(2.672));
        assert_eq!(ast::NumNode::new("2.672e3").num().unwrap().value, Numerics::Real(2672.0));
        assert_eq!(ast::NumNode::new("2.672e+16").num().unwrap().value, Numerics::Real(2.672 * 10f64.powf(16f64)));
        assert_eq!(ast::NumNode::new("2.672e-10").num().unwrap().value, Numerics::Real(2.672 * 10f64.powf(-10f64)));
        assert_eq!(ast::NumNode::new("67e-4").num().unwrap().value, Numerics::Real(0.0067));
        assert_eq!(ast::NumNode::new("67e+10").num().unwrap().value, Numerics::Natural(670000000000));
        assert_eq!(ast::NumNode::new("-2").num().unwrap().value, Numerics::Integer(-2));
        assert_eq!(ast::NumNode::new("-325").num().unwrap().value, Numerics::Integer(-325));
        assert_eq!(ast::NumNode::new("-0b01010110").num().unwrap().value, Numerics::Integer(-0b01010110));
        assert_eq!(ast::NumNode::new("-0o721").num().unwrap().value, Numerics::Integer(-0o721));
        assert_eq!(ast::NumNode::new("-0xfa").num().unwrap().value, Numerics::Integer(-250));
        assert_eq!(ast::NumNode::new("-0xf").num().unwrap().value, Numerics::Integer(-15));
        assert_eq!(ast::NumNode::new("-2.672").num().unwrap().value, Numerics::Real(-2.672));
        assert_eq!(ast::NumNode::new("-2.672e3").num().unwrap().value, Numerics::Real(-2672.0));
        assert_eq!(ast::NumNode::new("-2.672e+16").num().unwrap().value, Numerics::Real(-26720000000000000.0));
        assert_eq!(ast::NumNode::new("-2.672e-10").num().unwrap().value, Numerics::Real(-0.0000000002672));
        assert_eq!(ast::NumNode::new("-67e-4").num().unwrap().value, Numerics::Real(-0.0067));
        assert_eq!(ast::NumNode::new("-67e+10").num().unwrap().value, Numerics::Integer(-670000000000));

        let s : String = String::from("-6e12");
        let num = ast::NumNode::new(&*s);

        assert_eq!(num.num().unwrap().value, Numerics::Integer(-6000000000000));
    }
}