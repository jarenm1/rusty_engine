

type Entity = u32;


trait Component {}

// a component is a thing that we can assign to entties. But since entities are actually just a
// u32, that means that entities are really assignedt to components really??? components need to be
// de coupled from engine code so users can make custom components....  so component is a trait...
//
// since we are assigning entities to components. The component trait needs to have 


// commands.spawn() -> entity. what is commands??? commands is apart of our system
//
// ecs is row column based.. right. so we can imagine a table, the table has columns of components
// and rows of entities. column-x-axis row-y-axis
// 
// should we make components an option on an entity? does not seem memory efficient
//

#[derive(Default)]
pub struct System {
    next_entity: u32,
}

// macro to handle at compile time n arguments of type impl Component. 



// system needs to be the drivver for the ECS> has all the values for archetypes etc... its what i
// would look at for the table... we also need a way to spawn entities based on system. so like a
// system.spawn() ? but we could also do like a sub-type or a query index? 
// like how does query work? So all the entites are stored in different archetypes. different
// archetypes are composed of different components. so we could struct archetype { entities,
// components, } or something similar ??? /
//
impl System {
    pub fn new() -> Self {
        Self {
         next_entity: 0,
        }
    }
}

mod macros {
    macro_rules! spawn {
        (self:expr, $($component:expr), *) => {
            $(
                $self.next_entity += 1;
            )*
        };
    }
}
