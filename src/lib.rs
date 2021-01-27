// use std::convert::TryInto;
// use std::array::TryFromSliceError;

pub trait Parser {
    type Out;

    fn call(self, input: &str) -> Option<(Self::Out, &str)>;
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

    fn call(self, input: &str) -> Option<(Self::Out, &str)> {
        let f = self.func;
        self.parser.call(input).map(move |(a, b)| {
            (f(a), b)
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

    fn call(self, input: &str) -> Option<(Self::Out, &str)> {
        let f = self.func;
        self.parser.call(input).and_then(|(a, b)| {
            f(a).call(b)
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


impl<A> ParserOps<A> for dyn Parser<Out=A> {}


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


// pub struct IntItem {
//     endian: Endian
// }

// impl IntItem {
//     pub fn new(endian: Endian) -> Self {
//         Self { 
//             endian
//         }
//     }
// }

// impl Parser for IntItem {
//     type Out = i32;

//     fn call(self, bytes: &[u8]) -> Option<(Self::Out, &[u8])> {
//         self.endian.read_int(bytes).ok()
//     }
// }


// pub struct DoubleItem {
//     endian: Endian
// }

// impl DoubleItem {
//     pub fn new(endian: Endian) -> Self {
//         Self { 
//             endian
//         }
//     }
// }

// impl Parser for DoubleItem {
//     type Out = f64;

//     fn call(self, bytes: &[u8]) -> Option<(Self::Out, &[u8])> {
//         self.endian.read_double(bytes).ok()
//     }
// }

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

impl<P: Parser> Parser for Take<P> {
    type Out = Vec<<P as Parser>::Out>;

    fn call(self, input: &str) -> Option<(Self::Out, &str)> {
        // let init = Zero::<<P as Parser>::Out>::new().bind(|item| Return::new(item));
        // let init = Return::new(Vec::new());
        let init = Box::new(Return::new(Vec::new())); 
        (0..self.count)
            .map(|_| self.parser)
            .fold(init, |result, parser| {
                Box::new(result.bind(|vec| {
                    Box::new(parser.bind(|item| {
                        vec.push(item);
                        Return::new(vec)
                    }))
                }))
            })
            .call(input)
    }
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