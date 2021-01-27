// use std::convert::TryInto;
// use std::array::TryFromSliceError;

pub trait Parser {
    type Out;

    fn call(self, input: &str) -> Option<(Self::Out, &str)>;


    fn map<F, A>(self, func: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(<Self as Parser>::Out) -> A
    {
        Map::new(self, func)
    }

    fn bind<F, Q>(self, func: F) -> Bind<Self, F>
    where
        Self: Sized,
        Q: Parser,
        F: Fn(<Self as Parser>::Out) -> Q 
    {
        Bind::new(self, func)
    }
}

// https://doc.rust-lang.org/src/core/iter/traits/iterator.rs.html#97-3286
// https://doc.rust-lang.org/src/core/iter/adapters/mod.rs.html#884-887

#[derive(Copy, Clone)]
pub struct Map<P, F> {
    parser: P,
    func: F 
}

impl<P, F, A> Map<P, F>
where
    P: Parser,
    F: Fn(<P as Parser>::Out) -> A 
{
    pub fn new(parser: P, func: F) -> Self {
        Self {
            parser,
            func
        }
    }
}

impl<P, F, A> Parser for Map<P, F>  
where
    P: Parser,
    F: Fn(<P as Parser>::Out) -> A
{
    type Out = A;

    fn call(self, input: &str) -> Option<(Self::Out, &str)> {
        let f = self.func;
        self.parser.call(input).map(move |(a, b)| {
            (f(a), b)
        })
    }
}


#[derive(Copy, Clone)]
pub struct Bind<P, F> {
    parser: P,
    func: F
}

impl<P, F, Q> Bind<P, F> 
where
    P: Parser,
    Q: Parser,
    F: Fn(<P as Parser>::Out) -> Q,
{
    pub fn new(parser: P, func: F) -> Self {
        Self {
            parser,
            func
        }
    }
}

impl<P, F, Q> Parser for Bind<P, F> 
where
    P: Parser,
    Q: Parser,
    F: Fn(<P as Parser>::Out) -> Q 
{
    type Out = <Q as Parser>::Out;

    fn call(self, input: &str) -> Option<(Self::Out, &str)> {
        let f = self.func;
        self.parser.call(input).and_then(|(a, b)| {
            f(a).call(b)
        })
    }
}


// pub trait ParserOps : Parser {

//     fn map<F, A>(self, func: F) -> Map<Self, F>
//     where
//         Self: Sized,
//         F: Fn(<Self as Parser>::Out) -> A
//     {
//         Map::new(self, func)
//     }

//     fn bind<F, Q>(self, func: F) -> Bind<Self, F>
//     where
//         Self: Sized,
//         Q: Parser,
//         F: Fn(<Self as Parser>::Out) -> Q 
//     {
//         Bind::new(self, func)
//     }
// }


// impl<P: Parser> ParserOps for P {}

#[derive(Copy, Clone)]
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

    fn call(self, _input: &str) -> Option<(Self::Out, &str)> {
        None
    }
}

#[derive(Copy, Clone)]
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

    fn call(self, input: &str) -> Option<(A, &str)> {
        Some((self.data, input))
    }
}

#[derive(Copy, Clone)]
pub struct Item {

}

impl Item {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parser for Item {
    type Out = char;

    fn call(self, input: &str) -> Option<(Self::Out, &str)> {
        input
            .chars()
            .next()
            .map(|c| (c, &input[1..]))
    }
}

pub struct Take<P: Parser> {
    count: i32,
    parser: P
}

impl<P: Parser> Take<P> {
    pub fn new(count: i32, parser: P) -> Self {
        Self { count, parser }
    }
}

impl<P: Parser + Copy> Parser for Take<P> {
    type Out = Vec<<P as Parser>::Out>;

    fn call(self, input: &str) -> Option<(Self::Out, &str)> {
        
        let mut v = Vec::new();
        let mut rest = input;
        for _ in 0..self.count {
            match self.parser.call(rest) {
                None => return None,
                Some((item, string)) => {
                    v.push(item);
                    rest = string;
                }
            }
        }

        Some((v, rest))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_test() {
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
        let stuff = "hello world";

        let (result, _) = 
            Item::new()
                .bind(|x| Return::new(format!("hi, {}", x)))
                .call(&stuff)
                .unwrap();

        assert_eq!("hi, h", result);
    }

    #[test]
    fn take_test() {
        let stuff = "hello world";

        let (result, rest) = 
            Take::new(3, Item::new())
                .call(&stuff)
                .unwrap();

        assert_eq!(vec!['h', 'e', 'l'], result);
        assert_eq!("lo world", rest);
    }


    #[test]
    fn probably_not_needed_take_test() {
        let stuff = "hello world";

        let (result, rest) = 
            Take::new(3, Item::new())
                .map(|it| it.len()) 
                .call(&stuff)
                .unwrap();

        // assert_eq!(vec!['h', 'e', 'l'], result);
        assert_eq!(3, result);
        assert_eq!("lo world", rest);
    }
}