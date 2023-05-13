use dezorr::func::{Function, FunctionOn};

fn main() {
    let _v: Function<usize> = Function::coterminal(vec![0usize]);
    // println!("Hello, {:?}!", v.value().unwrap());
    let _f: Function<usize> = Function::<usize>::new(Some(Box::new(|x: usize| x + 1)), None);
    // v.propagate_value_to(&f);
    // println!("apply => {}", f.value().unwrap());
}
