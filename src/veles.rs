use glob::glob;
use glob::Paths;
use common::*;


/*
 * Veles is a service spawner deity
 */
pub struct Veles;


impl Veles {


    pub fn list_services() -> Paths {
        glob(
            &format!("{}/{}", SERVICES_DIR, SERVICES_GLOB)
        ).expect("Failed to match /Services/*.conf")
    }


}
