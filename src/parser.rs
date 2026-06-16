use std::env;

pub struct DjangoOptions {
    pub name: String,
    pub apps: String,
}

pub fn parser() -> Option<DjangoOptions> {
    let args: Vec<_> = env::args().collect();

    for mut arg in args.clone() {
        arg = arg.to_lowercase();
        if arg.contains("-h") {
            return None;
        }
    }
    match args.len() {
        2 => {
            if args[1].len() > 25 {
                panic!("Django Project name is too long!!!")
            }
            return Some(DjangoOptions {
                name: args[1].clone(),
                apps: String::from(""),
            });
        }
        3 => {
            if args[1].len() > 25 {
                panic!("Django Project name is too long!!!")
            }
            return Some(DjangoOptions {
                name: args[1].clone(),
                apps: args[2].clone(),
            });
        }
        _ => {
            return Some(DjangoOptions {
                name: String::from("core"),
                apps: String::from(""),
            });
        } // needs more work
    }
}
