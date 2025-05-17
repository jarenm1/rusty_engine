use engine::prelude::*;
use jaren_ecs::system::System;

fn main() {
    // add example
    todo!();
}

pub fn startup_function_1(system: &mut System) {
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&"Running startup system 1!".into());
    #[cfg(not(target_arch = "wasm32"))]
    println!("Running startup system 1!");
    // Example: register components, spawn entities, etc.
    let entity = system.arena.allocate();
    system.entity_to_archetype.insert(entity, (0, 0));
}

pub fn update_function_1(system: &mut System) {
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&"Running update system 1!".into());
    #[cfg(not(target_arch = "wasm32"))]
    println!("Running update system 1!");
    // Example: update logic, move entities, etc.
}
