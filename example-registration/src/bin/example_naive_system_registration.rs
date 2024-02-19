use crc32fast::hash;
use ctor::ctor;
use lazy_static::lazy_static;
use std::any::Any;
use std::sync::Mutex;
use std::cell::OnceCell;

// Component registration
lazy_static! {
    static ref COMPONENT_REGISTRY: Mutex<Vec<ComponentRegistryEntry>> = Mutex::new(vec![]);
}

#[derive(Debug)]
struct ComponentRegistryEntry {
    name: String,
    name_crc: u32,
    factory: fn() -> Box<dyn Any>,
    id: u32
}

macro_rules! register_component {
    ($component:ident, $function:ident) => {
        #[ctor]
        fn add_to_component_registry() {
            fn factory_any() -> Box<dyn Any> {
                let component = $function();
                Box::new(component)
            }
            let _ = COMPONENT_REGISTRY.lock().as_mut().and_then(|registry| {
                let id = registry.len() as u32;
                registry.push(ComponentRegistryEntry {
                name: stringify!($component).to_string(),
                name_crc: hash(stringify!($component).as_bytes()),
                factory: factory_any,
                id 
                });
                $component::set_id(id);
                Ok(())
            });
        }

        static COMPONENT_ID : Mutex<OnceCell<u32>> = Mutex::new(OnceCell::new());

        impl HasID for $component
        {
            fn get_id() -> u32
            {
                *COMPONENT_ID.lock().unwrap().get().expect("ID not yet set")
            }

            fn set_id(new_id : u32)
            {
                let _ = COMPONENT_ID
                    .lock()
                    .as_mut()
                    .map(
                        |id| 
                        {
                            let _ = id.set(new_id).expect("ID already set");
                        }
                    );
            }
        }
    };
}

pub trait HasID {

    fn get_id() -> u32;

    fn set_id(new_id : u32);
}

struct MyComponent {
    name: String,
}

fn create_component() -> MyComponent {
    MyComponent {
        name: "Default Name".to_string(),
    }
}


// System registration
lazy_static! {
    static ref SYSTEM_REGISTRY: Mutex<Vec<SystemRegistryEntry>> = Mutex::new(vec![]);
}

#[derive(Debug)]
struct SystemRegistryEntry {
    name: String,
    name_crc: u32,
    function: fn(Vec<Box<dyn Any>>) -> (),
    id: u32,
    dependencies: Vec<u32>
}

macro_rules! register_system {
    ($function:ident, dependencies = [$($dependency:ident),*]) => {
        #[ctor]
        fn add_to_system_registry() {
            let _ = SYSTEM_REGISTRY.lock().as_mut().and_then(|registry| {
                let dependencies : Vec<u32> = vec![$($dependency ::get_id() ),*];
                let id = registry.len() as u32;
                    registry.push(SystemRegistryEntry{
                    name: stringify!($function).to_string(),
                    name_crc: hash(stringify!($function).as_bytes()),
                    function: $function,
                    dependencies,
                    id 
                });
                Ok(())
            });
        }
    };
}

fn my_system (components : Vec<Box<dyn Any>>)
{
    let comp : &MyComponent = components[0].downcast_ref().unwrap();
    println!("My component is: {}", comp.name);
}

// Change the order of these two macros and you will have an execution order problem
register_system!(my_system, dependencies= [MyComponent]);
register_component! {
    MyComponent,
    create_component
}

fn main() {
    let comp_registry = COMPONENT_REGISTRY.lock().unwrap();
    for entry in comp_registry.iter() {
        let any_value = (entry.factory)();
        let component = any_value.downcast_ref::<MyComponent>().unwrap();
        // name: MyComponent, name_crc: 1359051788, id: 0
        println!("name: {}, name_crc: {}, id: {}", entry.name, entry.name_crc, entry.id);
        println!("Component name: {}", component.name);
    }

    let system_registry = SYSTEM_REGISTRY.lock().unwrap();
    for entry in system_registry.iter() {
        println!("System name: {}, id: {}, dependencies: {:?}", entry.name, entry.id, entry.dependencies);
    }
}
