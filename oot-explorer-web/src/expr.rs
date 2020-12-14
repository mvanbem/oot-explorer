use num_traits::identities::{One, Zero};
use slotmap::dense::DenseSlotMap;
use slotmap::DefaultKey;
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::{AddAssign, Index, MulAssign, Neg};

pub trait ValueType: Clone + Display + Eq + Hash + One + Zero {}
impl<T> ValueType for T where T: Clone + Display + Eq + Hash + One + Zero {}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Key(DefaultKey);

pub struct Context<T>
where
    T: ValueType,
{
    storage: DenseSlotMap<DefaultKey, Expr<T>>,
    index: HashMap<Expr<T>, DefaultKey>,
}
impl<T> Context<T>
where
    T: ValueType,
{
    pub fn new() -> Context<T> {
        Context {
            storage: DenseSlotMap::new(),
            index: HashMap::new(),
        }
    }
    pub fn get_with_ctx(&self, key: Key) -> Option<ExprWithContext<'_, T>> {
        self.storage
            .get(key.0)
            .map(|expr| ExprWithContext { expr, ctx: self })
    }
    fn intern(&mut self, expr: Expr<T>) -> Key {
        if let Some(key) = self.index.get(&expr) {
            Key(*key)
        } else {
            let key = self.storage.insert(expr.clone());
            self.index.insert(expr, key);
            Key(key)
        }
    }
    pub fn literal(&mut self, value: T) -> Key {
        self.intern(Expr::Literal(value))
    }
    pub fn symbol(&mut self, text: String) -> Key {
        self.intern(Expr::Symbol(text))
    }
}
impl<T> Context<T>
where
    T: AddAssign<T> + ValueType,
{
    pub fn add(&mut self, keys: Vec<Key>) -> Key {
        let mut todo = keys;
        let mut new_keys = vec![];
        let mut literal_sum = T::zero();
        while let Some(key) = todo.pop() {
            match &self[key] {
                Expr::Literal(value) => literal_sum += value.clone(),
                Expr::Add(keys) => todo.extend_from_slice(keys),
                _ => new_keys.push(key),
            }
        }
        if !literal_sum.is_zero() {
            new_keys.push(self.literal(literal_sum));
        }
        match new_keys.len() {
            // An empty sum is zero.
            0 => self.literal(Zero::zero()),
            // A singleton sum is just the given term.
            1 => new_keys[0],
            _ => {
                // Normal form: terms are sorted by their slotmap keys.
                new_keys.sort_unstable();
                self.intern(Expr::Add(new_keys))
            }
        }
    }
}
impl<T> Context<T>
where
    T: MulAssign<T> + ValueType,
{
    pub fn mul(&mut self, keys: Vec<Key>) -> Key {
        let mut todo = keys;
        let mut new_keys = vec![];
        let mut literal_product = T::one();
        while let Some(key) = todo.pop() {
            match &self[key] {
                Expr::Literal(value) => literal_product *= value.clone(),
                Expr::Mul(keys) => todo.extend_from_slice(keys),
                _ => new_keys.push(key),
            }
        }
        match literal_product {
            x if x.is_zero() => return self.literal(x),
            x if !x.is_one() => new_keys.push(self.literal(x)),
            _ => (),
        }
        match new_keys.len() {
            // An empty product is one.
            0 => self.literal(One::one()),
            // A singleton product is just the given term.
            1 => new_keys[0],
            _ => {
                // Normal form: terms are sorted by their slotmap keys.
                new_keys.sort_unstable();
                self.intern(Expr::Mul(new_keys))
            }
        }
    }
}
impl<T> Context<T>
where
    T: Neg<Output = T> + ValueType,
{
    pub fn neg(&mut self, key: Key) -> Key {
        match &self[key] {
            // Negation reaches into literals.
            Expr::Literal(value) => {
                let neg_value = -value.clone();
                self.intern(Expr::Literal(neg_value))
            }
            // Double negation cancels out.
            Expr::Neg(key) => *key,
            _ => self.intern(Expr::Neg(key)),
        }
    }
}
impl<T> Index<Key> for Context<T>
where
    T: ValueType,
{
    type Output = Expr<T>;
    fn index(&self, key: Key) -> &Expr<T> {
        &self.storage[key.0]
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Expr<T>
where
    T: ValueType,
{
    Literal(T),
    Symbol(String),
    Neg(Key),
    Add(Vec<Key>),
    Mul(Vec<Key>),
}
impl<T> Expr<T>
where
    T: ValueType,
{
    fn precedence(&self) -> Precedence {
        match self {
            Expr::Literal(_) => Precedence::Atomic,
            Expr::Symbol(_) => Precedence::Atomic,
            Expr::Neg(_) => Precedence::Neg,
            Expr::Add(_) => Precedence::Add,
            Expr::Mul(_) => Precedence::Mul,
        }
    }
    fn fmt(
        &self,
        ctx: &Context<T>,
        context_precedence: Precedence,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        let precedence = self.precedence();
        let parens = precedence < context_precedence;
        if parens {
            write!(f, "(")?;
        }
        match self {
            Expr::Literal(value) => write!(f, "{}", value)?,
            Expr::Symbol(text) => write!(f, "{}", text)?,
            Expr::Neg(key) => {
                write!(f, "-")?;
                ctx[*key].fmt(ctx, precedence, f)?;
            }
            Expr::Add(keys) => {
                let mut pos_sep = "";
                let mut neg_sep = "-";
                for key in keys {
                    match &ctx[*key] {
                        Expr::Neg(key) => {
                            // Special case: use a minus sign.
                            write!(f, "{}", neg_sep)?;
                            ctx[*key].fmt(ctx, precedence, f)?;
                        }
                        expr => {
                            write!(f, "{}", pos_sep)?;
                            expr.fmt(ctx, precedence, f)?;
                        }
                    }
                    pos_sep = " + ";
                    neg_sep = " - ";
                }
            }
            Expr::Mul(keys) => {
                let mut sep = "";
                for key in keys {
                    write!(f, "{}", sep)?;
                    sep = " * ";
                    ctx[*key].fmt(ctx, precedence, f)?;
                }
            }
        }
        if parens {
            write!(f, ")")?;
        }
        Ok(())
    }
}

pub struct ExprWithContext<'a, T>
where
    T: ValueType,
{
    expr: &'a Expr<T>,
    ctx: &'a Context<T>,
}
impl<'a, T> std::fmt::Display for ExprWithContext<'a, T>
where
    T: ValueType,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.expr.fmt(self.ctx, Precedence::Add, f)
    }
}
impl<'a, T> std::fmt::Debug for ExprWithContext<'a, T>
where
    T: ValueType,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.expr.fmt(self.ctx, Precedence::Add, f)
    }
}
impl<'a, T> PartialEq for ExprWithContext<'a, T>
where
    T: ValueType,
{
    fn eq(&self, other: &ExprWithContext<'a, T>) -> bool {
        ((self.expr as *const Expr<T>) == (other.expr as *const Expr<T>))
            && ((self.ctx as *const Context<T>) == (other.ctx as *const Context<T>))
    }
}
impl<'a, T> Eq for ExprWithContext<'a, T> where T: ValueType {}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum Precedence {
    Add,
    Mul,
    Neg,
    Atomic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_dedupes() {
        let mut ctx = Context::<u8>::new();
        let five_a = ctx.literal(5);
        let five_b = ctx.literal(5);
        assert_eq!(five_a, five_b);
    }

    #[test]
    fn symbol_dedupes() {
        let mut ctx = Context::<u8>::new();
        let x_a = ctx.symbol("x".to_string());
        let x_b = ctx.symbol("x".to_string());
        assert_eq!(x_a, x_b);
    }

    #[test]
    fn double_negation_cancels() {
        let mut ctx = Context::<i8>::new();
        let x = ctx.symbol("x".to_string());
        let neg_x = ctx.neg(x);
        let neg_neg_x = ctx.neg(neg_x);
        assert_eq!(x, neg_neg_x);
    }

    #[test]
    fn neg_literal() {
        let mut ctx = Context::<i8>::new();
        let zero = ctx.literal(0);
        let neg_zero = ctx.neg(zero);
        assert_eq!(zero, neg_zero);
        let literal_neg_five = ctx.literal(-5);
        let five = ctx.literal(5);
        let neg_five = ctx.neg(five);
        assert_eq!(literal_neg_five, neg_five);
    }

    #[test]
    fn add_none_is_zero() {
        let mut ctx = Context::<u8>::new();
        let zero = ctx.literal(0);
        let sum = ctx.add(vec![]);
        assert_eq!(zero, sum);
    }

    #[test]
    fn add_singleton_reduces() {
        let mut ctx = Context::<u8>::new();
        let x = ctx.symbol("x".to_string());
        let sum = ctx.add(vec![x]);
        assert_eq!(x, sum);
    }

    #[test]
    fn add_is_order_independent() {
        let mut ctx = Context::<u8>::new();
        let x = ctx.symbol("x".to_string());
        let y = ctx.symbol("y".to_string());
        let sum_a = ctx.add(vec![x, y]);
        let sum_b = ctx.add(vec![y, x]);
        assert_eq!(sum_a, sum_b);
    }

    #[test]
    fn add_merges_literals() {
        let mut ctx = Context::<u8>::new();
        let x = ctx.symbol("x".to_string());
        let y = ctx.symbol("y".to_string());
        let three = ctx.literal(3);
        let five = ctx.literal(5);
        let eight = ctx.literal(8);
        let sum_a = ctx.add(vec![x, y, three, five]);
        let sum_b = ctx.add(vec![x, y, eight]);
        assert_eq!(sum_a, sum_b);
    }

    #[test]
    fn add_drops_zero() {
        let mut ctx = Context::<i8>::new();
        let x = ctx.symbol("x".to_string());
        let three = ctx.literal(3);
        let neg_three = ctx.literal(-3);
        let sum = ctx.add(vec![x, three, neg_three]);
        assert_eq!(x, sum);
    }

    #[test]
    fn add_flattens() {
        let mut ctx = Context::<i8>::new();
        let x = ctx.symbol("x".to_string());
        let y = ctx.symbol("y".to_string());
        let z = ctx.symbol("z".to_string());
        let xy = ctx.add(vec![x, y]);
        let xyz_a = ctx.add(vec![xy, z]);
        let xyz_b = ctx.add(vec![x, y, z]);
        assert_eq!(
            ctx.get_with_ctx(xyz_a).unwrap(),
            ctx.get_with_ctx(xyz_b).unwrap(),
        );
    }

    #[test]
    fn mul_none_is_one() {
        let mut ctx = Context::<u8>::new();
        let one = ctx.literal(1);
        let product = ctx.mul(vec![]);
        assert_eq!(one, product);
    }

    #[test]
    fn mul_singleton_reduces() {
        let mut ctx = Context::<u8>::new();
        let x = ctx.symbol("x".to_string());
        let product = ctx.mul(vec![x]);
        assert_eq!(x, product);
    }

    #[test]
    fn mul_is_order_independent() {
        let mut ctx = Context::<u8>::new();
        let x = ctx.symbol("x".to_string());
        let y = ctx.symbol("y".to_string());
        let product_a = ctx.mul(vec![x, y]);
        let product_b = ctx.mul(vec![y, x]);
        assert_eq!(product_a, product_b);
    }

    #[test]
    fn mul_merges_literals() {
        let mut ctx = Context::<u8>::new();
        let x = ctx.symbol("x".to_string());
        let y = ctx.symbol("y".to_string());
        let three = ctx.literal(3);
        let five = ctx.literal(5);
        let fifteen = ctx.literal(15);
        let product_a = ctx.mul(vec![x, y, three, five]);
        let product_b = ctx.mul(vec![x, y, fifteen]);
        assert_eq!(product_a, product_b);
    }

    #[test]
    fn mul_drops_one() {
        let mut ctx = Context::<i8>::new();
        let x = ctx.symbol("x".to_string());
        let one = ctx.literal(1);
        let product = ctx.mul(vec![x, one]);
        assert_eq!(x, product);
    }

    #[test]
    fn mul_zero_dominates() {
        let mut ctx = Context::<i8>::new();
        let x = ctx.symbol("x".to_string());
        let zero = ctx.literal(0);
        let product = ctx.mul(vec![x, zero]);
        assert_eq!(zero, product);
    }

    #[test]
    fn mul_flattens() {
        let mut ctx = Context::<i8>::new();
        let x = ctx.symbol("x".to_string());
        let y = ctx.symbol("y".to_string());
        let z = ctx.symbol("z".to_string());
        let xy = ctx.mul(vec![x, y]);
        let xyz_a = ctx.mul(vec![xy, z]);
        let xyz_b = ctx.mul(vec![x, y, z]);
        assert_eq!(xyz_a, xyz_b);
    }
}
