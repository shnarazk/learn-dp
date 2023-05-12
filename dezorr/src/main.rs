use dezorr::func::{Function, FunctionOn};

fn main() {
    let v: Function<usize> = Function::constant(0usize);
    println!("Hello, {:?}!", v.value().unwrap());
    let f: Function<usize> = Function::<usize>::new(Box::new(|x: usize| x + 1));
    v.propagate_value_to(&f);
    println!("apply => {}", f.value().unwrap());
}
