use dezorr::{func::Function, var::Variable};

fn main() {
    let v: Variable<usize> = Variable::new(0usize);
    println!("Hello, {:?}!", v.data);
    let f: Function<usize> = Function::<usize>::new(Box::new(|x: usize| x + 1));
    println!("apply => {}", f.apply(v).data)
}
