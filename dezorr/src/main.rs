use dezorr::{
    func::{Function, FunctionOn},
    DFN,
};

fn main() {
    let _v: Function<usize> = Function::<usize>::coterminal(vec![0]);
    // println!("Hello, {:?}!", v.value().unwrap());
    let _f: Function<usize> = Function::<usize>::new(DFN!(|x| x + 1), None);
    // v.propagate_value_to(&f);
    // println!("apply => {}", f.value().unwrap());
}
