mod starter;
mod check;
mod parser;
fn main(){

    let res = parser::parser();
    match res {
        Some(options) => starter::starter(options),
        None => println!("Failed to parse arguments"),
    }
}