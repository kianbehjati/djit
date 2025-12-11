use reqwest::blocking::Client;

pub fn checker() -> bool{
    let response = Client::new().get("https://google.com").send();
    match response {
        Ok(_) => {
            return true;
        },
        Err(err) => {
            println!("{:?}",err);
            println!("Please Check your Connection...");
            return false;
        },
    }
}
