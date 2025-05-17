use std::{any::{Any, TypeId}, collections::HashMap};

pub type Entity = u64;

pub trait Component: Any + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Default)]
pub struct World {
    next_entity: Entity,
    archetypes: Vec<Archetype>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            archetypes: Vec::new(),
        }
    }
}

pub struct Archetype {
    // components are a vec of anything that implements component
    components: HashMap<TypeId, Vec<Box<dyn Component>>>,
    entities: Vec<Entity>,
}

impl Archetype {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            entities: Vec::new(),
        }
    }
}

pub trait SystemFn<World> {
    fn run(&mut self, world: &mut World);
}

pub struct System<F, A> {
    func: F,
    _marker: std::marker::PhantomData<A>,
}

// Implement for function pointer types
impl<F, A> SystemFn<World> for System<F, A>
where
    for<'a> F: FnMut(<A as SystemParam>::Param<'a>) + 'static,
    A: SystemParam,
{
    fn run(&mut self, world: &mut World) {
        let param = A::fetch(world);
        (self.func)(param);
    }
}

// SystemParam trait for argument extraction
pub trait SystemParam {
    type Param<'a>;
    fn fetch<'a>(world: &'a mut World) -> Self::Param<'a>;
}

pub struct Scheduler {
    systems: Vec<Box<dyn SystemFn<World>>>,
}

// Scheduler is the driver for the ECS. It is responsible for running systems.
// Need to implement something to prevent the edge case of functions being run out of order causing unexpected behavior.
impl Scheduler {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }
    pub fn add_system<F: SystemFn<World> + 'static>(&mut self, system: F) {
        self.systems.push(Box::new(system));
    }
    pub fn run(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system.run(world);
        }
    }
}

pub struct Query<'a, T> {
    world: &'a World,
    _marker: std::marker::PhantomData<T>,
}

/// Implement Query for single component queries
impl<'a, T: Component> Query<'a, T> {
    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.world.archetypes.iter().flat_map(|archetype| {
            // Get the Vec for T in this archetype
            let vec = match archetype.components.get(&std::any::TypeId::of::<T>()) {
                Some(vec) => vec,
                None => return Vec::new().into_iter(),
            };
            // Iterate by index, matching with entities
            (0..archetype.entities.len()).filter_map(move |i| {
                let t_ref = vec[i].as_any().downcast_ref::<T>()?;
                Some((archetype.entities[i], t_ref))
            }).collect::<Vec<_>>().into_iter()
        })
    }
}

macro_rules! tuple_refs {
    ($a:ident, $b:ident $(, $rest:ident)*) => {
        (& $a, & $b $(, & $rest)*)
    };
}

/// Implement Query for tuple component queries
macro_rules! impl_query_iter_tuple {
    ($a:ident, $b:ident) => {
        impl<'a, $a: Component, $b: Component> Query<'a, ($a, $b)> {
            pub fn iter(&self) -> Box<dyn Iterator<Item = (Entity, (&$a, &$b))> + '_> {
                Box::new(self.world.archetypes.iter().flat_map(|archetype| {
                    let $a = match archetype.components.get(&std::any::TypeId::of::<$a>()) {
                        Some(vec) => vec,
                        None => return Box::new(std::iter::empty()) as Box<dyn Iterator<Item = (Entity, (&$a, &$b))> + '_>,
                    };
                    let $b = match archetype.components.get(&std::any::TypeId::of::<$b>()) {
                        Some(vec) => vec,
                        None => return Box::new(std::iter::empty()) as Box<dyn Iterator<Item = (Entity, (&$a, &$b))> + '_>,
                    };
                    Box::new((0..archetype.entities.len()).filter_map(move |i| {
                        Some((
                            archetype.entities[i],
                            (
                                $a[i].as_any().downcast_ref::<$a>()?,
                                $b[i].as_any().downcast_ref::<$b>()?,
                            )
                        ))
                    })) as Box<dyn Iterator<Item = (Entity, (&$a, &$b))> + '_>
                }))
            }
        }
    };
}
impl_query_iter_tuple!(T1, T2);

// Implement SystemParam for queries, resources, etc.
impl<T: Component> SystemParam for Query<'_, T> {
    type Param<'a> = Query<'a, T>;
    fn fetch<'a>(world: &'a mut World) -> Query<'a, T> {
        Query {
            world,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Implement SystemParam for tuple queries
macro_rules! impl_system_param_for_query_tuple {
    ($( $name:ident ),*) => {
        impl<'a, $( $name: Component ),*> SystemParam for Query<'a, ( $( $name ),* )> {
            type Param<'b> = Query<'b, ( $( $name ),* )>;
            fn fetch<'b>(world: &'b mut World) -> Query<'b, ( $( $name ),+ )> {
                Query {
                    world,
                    _marker: std::marker::PhantomData,
                }
            }
        }
    }
}

impl_system_param_for_query_tuple!(T1, T2);
impl_system_param_for_query_tuple!(T1, T2, T3);
impl_system_param_for_query_tuple!(T1, T2, T3, T4);

#[macro_export]
/// Spawn a new entity with the given components. 
/// 
/// # Note
/// All arguments must implement the [Component](trait.Component.html) trait.
/// If you pass a type that does not implement the trait, you will get a compile error.
/// 
/// # Example
/// ```
/// #[derive(Component)]
/// struct Position(f32, f32);
/// 
/// let entity = spawn!(world, Position(0.0, 0.0));
/// ```
macro_rules! spawn {
    ($world:expr, $($component:expr),*) => {{
        let entity = $world.next_entity;
        $world.next_entity = $world.next_entity.wrapping_add(1);
        let components_vec: Vec<Box<dyn Component>> = vec![$(Box::new($component)),*];
        $world.get_archetype(entity, components_vec);
        entity
    }};
}

impl World {
    /// Find or create an archetype for a set of components.
    pub fn get_archetype(&mut self, entity: Entity, components: Vec<Box<dyn Component>>) {
        use std::any::TypeId;
        // Get the set of component types for this entity
        let incoming_types: Vec<TypeId> = components.iter().map(|c| c.as_any().type_id()).collect();

        // Find an archetype with exactly this set of component types
        let archetype = self.archetypes.iter_mut().find(|archetype| {
            let archetype_types: Vec<TypeId> = archetype.components.keys().cloned().collect();
            incoming_types.len() == archetype_types.len()
                && incoming_types.iter().all(|t| archetype_types.contains(t))
        });

        if let Some(archetype) = archetype {
            archetype.entities.push(entity);
            for component in components {
                let type_id = component.as_any().type_id();
                archetype.components.entry(type_id).or_default().push(component);
            }
        } else {
            let mut archetype = Archetype::new();
            archetype.entities.push(entity);
            for component in components {
                let type_id = component.as_any().type_id();
                archetype.components.entry(type_id).or_default().push(component);
            }
            self.archetypes.push(archetype);
        }
    }
}

#[cfg(test)]
mod tests {
    use jaren_ecs_derive::Component;

    use super::*;

    #[derive(Component)]
    struct Position(f32, f32);

    #[derive(Component)]
    struct Player;

    #[test]
    fn test_entity_creation() {
        let mut world = World::new();
        let entity = spawn!(world, Position(0.0, 0.0));
        let entity2 = spawn!(world, Position(1.0, 0.0));
        assert_eq!(entity, 0);
        assert_eq!(entity2, 1);
    }

    #[test]
    fn test_query() {
        let mut world = World::new();
        let entity = spawn!(world, Position(0.0, 0.0));
        let entity2 = spawn!(world, Position(1.0, 0.0));
        let query = Query::<Position> { world: &world, _marker: std::marker::PhantomData };
        let results = query.iter().collect::<Vec<_>>();
        assert_eq!(results[0].0, entity);
        assert_eq!(results[1].0, entity2);
    }
    #[test]
    fn test_query_tuple() {
        let mut world = World::new();
        let entity = spawn!(world, Position(0.0, 0.0), Player);
        let entity2 = spawn!(world, Position(1.0, 0.0));
        let query = Query::<(Position, Player)> { world: &world, _marker: std::marker::PhantomData };
        let results = query.iter().collect::<Vec<_>>();
        assert_eq!(results[0].0, entity);
        assert_eq!(results[0].0, entity);
    }

}
