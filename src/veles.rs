// use service::Service;
use glob::glob;
use glob::Paths;


/*
 * Veles is a service spawner deity
 */
pub struct Veles;


impl Veles {


    pub fn list_services() -> Paths {
        glob("/Services/*.conf").expect("Failed to match /Services/*.conf")
    }


}
