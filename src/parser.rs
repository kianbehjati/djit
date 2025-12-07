use std::env;

pub struct DjangoOptions {
    pub name:String,
    pub apps:String,
}

pub fn parser() -> DjangoOptions<>{
    let args:Vec<_> = env::args().collect();
    match args.len() {
        2 => {
            return DjangoOptions{name : args[1].clone(),apps : String::from("")};
           
        }
        3 => {
            return DjangoOptions{name : args[1].clone(),apps : args[2].clone()};
        }
        _ => {
            return DjangoOptions{name : String::from("core"),apps : String::from("")};
        }
        // needs more work
    }
   
}