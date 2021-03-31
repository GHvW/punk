// use std::convert::TryInto;
// use std::array::TryFromSliceError;

pub trait Parser {
    type Out;

    fn call<'a>(&self, input: &'a str) -> Option<(Self::Out, &'a str)>;

    // fn map<F, B>(self, func: F) -> Map<F, Self::Out>
    // where
    //     Self: Sized + 'static,
    //     F: Fn(Self::Out) -> B
    // {
    //     Map::new(Box::new(self), func)
    // }

    // fn bind<F, B>(self, func: F) -> Bind<F, Self::Out>
    // where
    //     Self: Sized + 'static,
    //     F: Fn(Self::Out) -> Box<dyn Parser<Out=B>>
    // {
    //     Bind::new(Box::new(self), func)
    // }
}

// https://doc.rust-lang.org/src/core/iter/traits/iterator.rs.html#97-3286
// https://doc.rust-lang.org/src/core/iter/adapters/mod.rs.html#884-887

pub struct Map<F, A> {
    parser: Box<dyn Parser<Out=A>>,
    func: F 
}

impl<F, A, B> Map<F, A>
where
    F: Fn(A) -> B 
{
    pub fn new(parser: Box<dyn Parser<Out=A>>, func: F) -> Self {
        Self {
            parser,
            func
        }
    }
}

impl<F, A, B> Parser for Map<F, A>  
where
    F: Fn(A) -> B
{
    type Out = B;

    fn call<'a>(&self, input: &'a str) -> Option<(Self::Out, &'a str)> {
        self.parser.call(input).map(|(a, b)| {
            ((self.func)(a), b)
        })
    }
}


pub struct Bind<F, A> {
    parser: Box<dyn Parser<Out=A>>,
    func: F
}

impl<F, A, B> Bind<F, A> 
where
    F: Fn(A) -> Box<dyn Parser<Out=B>>,
{
    pub fn new(parser: Box<dyn Parser<Out=A>>, func: F) -> Self {
        Self {
            parser,
            func
        }
    }
}

impl<F, A, B> Parser for Bind<F, A> 
where
    F: Fn(A) -> Box<dyn Parser<Out=B>>
{
    type Out = B;

    fn call<'a>(&self, input: &'a str) -> Option<(Self::Out, &'a str)> {
        self.parser.call(input).and_then(|(a, b)| {
            (self.func)(a).call(b)
        })
    }
}


pub trait ParserOps<A> : Parser<Out=A> {

    fn map<F, B>(self, func: F) -> Map<F, A>
    where
        Self: Sized + 'static,
        F: Fn(A) -> B
    {
        Map::new(Box::new(self), func)
    }

    fn bind<F, B>(self, func: F) -> Bind<F, A>
    where
        Self: Sized + 'static,
        F: Fn(A) -> Box<dyn Parser<Out=B>>
    {
        Bind::new(Box::new(self), func)
    }
}


impl<A> ParserOps<A> for A where A: Parser<Out=A> {}


pub struct Zero<A> {
    phantom: std::marker::PhantomData<A>
}

impl<A> Zero<A> {
    pub fn new() -> Self { 
        Self { phantom: std::marker::PhantomData } 
    }
}

impl<A> Parser for Zero<A> {
    type Out = A;

    fn call<'a>(&self, _input: &'a str) -> Option<(Self::Out, &'a str)> {
        None
    }
}


pub struct Return<A> {
    data: A
}

impl<A> Return<A> {
    pub fn new(data: A) -> Self {
        Self { data }
    }
}

impl<A> Parser for Return<A> {
    type Out = A;

    fn call<'a>(&self, input: &'a str) -> Option<(A, &'a str)> {
        Some((self.data, input))
    }
}


pub struct Item {}

impl Item {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parser for Item {
    type Out = char;

    fn call<'a>(&self, input: &'a str) -> Option<(Self::Out, &'a str)> {
        input
            .chars()
            .next()
            .map(|c| (c, &input[1..]))
    }
}

pub struct Take<A> {
    count: i32,
    parser: Box<dyn Parser<Out=A>> 
}

impl<A> Take<A> {
    pub fn new(count: i32, parser: Box<dyn Parser<Out=A>>) -> Self {
        Self { count, parser }
    }
}

impl<A> Parser for Take<A> {
    type Out = Vec<A>;

    fn call<'a>(&self, input: &'a str) -> Option<(Self::Out, &'a str)> {
        // let init = Zero::<<P as Parser>::Out>::new().bind(|item| Return::new(item));
        // let init = Return::new(Vec::new());
        let init = Return::new(Vec::new()); 
        (0..self.count)
            .fold(init, |result, _| {
                result.bind(|a| {
                    (self.parser)(a)
                })
            })
            .call(input)
    }
}

fn take_reduce<A>(agg: Box<dyn Parser<Out=Vec<Box<A>>>>, next: Box<dyn Parser<Out=A>>) -> impl Parser<Out=Vec<Box<A>>> {
    agg.bind(|v| {
        Box::new(next.bind(|a| {                
            v.push(Box::new(a));
            Box::new(Return::new(v))
        }))
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_test() {
        // let stuff = [0b00000000, 0b00000000, 0b00100011, 0b00101000];
        let stuff = "hello world";

        let (result, _) = 
            Item::new()
                .map(|x| format!("hi, {}", x))
                .call(&stuff)
                .unwrap();

        assert_eq!("hi, h", result);
    }

    #[test]
    fn bind_test() {
        // let stuff = [0b00000000, 0b00000000, 0b00100011, 0b00101000];
        let stuff = "hello world";

        let (result, _) = 
            Item::new()
                .bind(|x| Return::new(format!("hi, {}", x)))
                .call(&stuff)
                .unwrap();

        assert_eq!("hi, h", result);
    }
}