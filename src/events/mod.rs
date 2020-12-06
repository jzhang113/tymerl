use rltk::Point;
use specs::prelude::*;
use std::sync::Mutex;

lazy_static! {
    pub static ref STACK: Mutex<Vec<Event>> = Mutex::new(Vec::new());
}

#[derive(Debug)]
pub enum EventType {
    Damage { amount: i32 },
}

pub struct Event {
    pub event_type: EventType,
    pub source: Option<Entity>,
    pub target_tiles: Vec<Point>,
    pub invokes_reaction: bool,
}

pub fn add_event(targets: Vec<Point>) {
    STACK.lock().expect("Failed to lock STACK").push(Event {
        event_type: EventType::Damage { amount: 1 },
        source: None,
        target_tiles: targets,
        invokes_reaction: true,
    });
}

pub fn process_stack(ecs: &mut World) {
    loop {
        let event = STACK.lock().expect("Failed to lock STACK").pop();
        match event {
            None => {
                break;
            }
            Some(ev) => {
                if ev.target_tiles.is_empty() {
                    // non-targetted events
                    process_event(ecs, &ev);
                } else {
                    let mut entities_hit = get_affected_entities(ecs, &ev.target_tiles);
                    entities_hit.retain(|ent| entity_can_react(ecs, ent));

                    // check if there are entities that can respond
                    if ev.invokes_reaction && !entities_hit.is_empty() {
                        let mut can_act = ecs.write_storage::<super::CanActFlag>();

                        for entity in entities_hit {
                            can_act
                                .insert(entity, super::CanActFlag { is_reaction: true })
                                .expect("Failed to insert CanActFlag");
                        }

                        // put the event back on the stack and return control to the main loop
                        STACK.lock().expect("Failed to lock STACK").push(ev);
                        break;
                    } else {
                        // otherwise resolve the event
                        process_event(ecs, &ev);
                    }
                }
            }
        }
    }
}

fn get_affected_entities(ecs: &mut World, targets: &Vec<Point>) -> Vec<Entity> {
    let mut affected = Vec::new();
    let positions = ecs.read_storage::<super::Position>();
    let entities = ecs.entities();

    for (ent, pos) in (&entities, &positions).join() {
        for target in targets {
            if pos.as_point() == *target {
                affected.push(ent);
            }
        }
    }

    affected
}

fn entity_can_react(ecs: &mut World, target: &Entity) -> bool {
    let can_react = ecs.read_storage::<super::CanReactFlag>();
    can_react.get(*target).is_some()
}

// placeholder handling
fn process_event(ecs: &mut World, event: &Event) {
    match event.event_type {
        EventType::Damage { amount } => {
            {
                let mut builder = ecs.fetch_mut::<super::ParticleBuilder>();

                for pos in &event.target_tiles {
                    builder.request(
                        *pos,
                        rltk::RGB::named(rltk::RED),
                        rltk::to_cp437('█'),
                        600.0,
                    );
                }
            }

            for ent in get_affected_entities(ecs, &event.target_tiles) {
                println!("{:?} took {:?} damage from {:?}", ent, amount, event.source)
            }
        }
    }
}
