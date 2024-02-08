use crc32fast::hash;
use ctor::ctor;
use lazy_static::lazy_static;
use std::any::Any;
use std::sync::Mutex;

lazy_static! {
    static ref COMPONENT_REGISTERS: Mutex<Vec<RegisterEntry>> = Mutex::new(vec![]);
}

#[derive(Debug)]
struct RegisterEntry {
    name: String,
    name_crc: u32,
    factory: fn() -> Box<dyn Any>,
}

macro_rules! register_component {
    ($component:ident, $function:ident) => {
        #[ctor]
        fn add_to_register() {
            fn factory_any() -> Box<dyn Any> {
                let component = $function();
                Box::new(component)
            }
            COMPONENT_REGISTERS.lock().unwrap().push(RegisterEntry {
                name: stringify!($component).to_string(),
                name_crc: hash(stringify!($component).as_bytes()),
                factory: factory_any,
            });
        }
    };
}

struct MyComponent {
    name: String,
}

fn create_component() -> MyComponent {
    MyComponent {
        name: "Default Name".to_string(),
    }
}

register_component! {
    MyComponent,
    create_component
}

fn main() {
    let registry = COMPONENT_REGISTERS.lock().unwrap();
    for entry in registry.iter() {
        let any_value = (entry.factory)();
        let component = any_value.downcast_ref::<MyComponent>().unwrap();
        println!("name: {}, name_crc: {}", entry.name, entry.name_crc);
        println!("Component name: {}", component.name);
    }
}
