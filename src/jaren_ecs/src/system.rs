use std::any::TypeId;
use std::collections::HashMap;

trait Component: 'static {}

// Component metadata for type-erased storage
struct ComponentMeta {
    size: usize,
    align: usize,
}

// Component Slice stores components as raw bytes.
struct ComponentSlice {
    data: Vec<u8>,
    count: usize,
}

// Archetype identifier
type ArchetypeId = u64;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Entity {
    id: u64,
    generation: u32,
}

struct Archetype {
    id: ArchetypeId,
    components: Vec<ComponentSlice>,
    entity_indices: Vec<usize>,
}

struct GenerationalArena {
    entities: Vec<Option<(EntityData, u32)>>,
    next_id: u64,
}

struct EntityData {
    archetype_id: ArchetypeId,
    index: usize,
}

pub struct System {
    component_registry: HashMap<TypeId, ComponentMeta>,
    archetypes: HashMap<Archetype, Archetype>,
    entity_to_archetype: HashMap<Entity, (ArchetypeId, usize)>,
    arena: GenerationalArena,
}

impl GenerationalArena {
    fn new() -> Self {
        Self {
            entities: Vec::new(),
            next_id: 0,
        }
    }

    fn allocate(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        self.entities.push(Some((
            EntityData {
                archetype_id: 0,
                index: 0,
            },
            0,
        )));
        Entity { id, generation: 0 }
    }
}

impl System {
    pub fn new() -> Self {
        Self {
            component_registry: HashMap::new(),
            archetypes: HashMap::new(),
            entity_to_archetype: HashMap::new(),
            arena: GenerationalArena::new(),
        }
    }
}
