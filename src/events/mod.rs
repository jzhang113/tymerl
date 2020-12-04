use specs::prelude::*;
use std::sync::Mutex;

lazy_static! {
    pub static ref STACK: Mutex<Vec<Event>> = Mutex::new(Vec::new());
}

pub enum EventType {
    Damage { amount: i32 },
}

pub struct Event {
    pub event_type: EventType,
    pub source: Option<Entity>,
    pub target: Option<Entity>,
    pub invokes_reaction: bool,
}

pub fn add_event(target: Option<Entity>) {
    STACK.lock().expect("Failed to lock STACK").push(Event {
        event_type: EventType::Damage { amount: 1 },
        source: None,
        target,
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
            Some(ev) => match ev.target {
                None => process_event(ecs, &ev),
                Some(target) => {
                    let entities_hit = get_reactions(ecs, &target);
                    if ev.invokes_reaction && !entities_hit.is_empty() {
                        let mut can_act = ecs.write_storage::<super::CanActFlag>();

                        for entity in entities_hit {
                            can_act
                                .insert(entity, super::CanActFlag { is_reaction: true })
                                .expect("Failed to insert CanActFlag");
                        }

                        // put the event back on the stack
                        STACK.lock().expect("Failed to lock STACK").push(ev);

                        break;
                    } else {
                        process_event(ecs, &ev);
                    }
                }
            },
        }
    }
}

fn get_reactions(ecs: &mut World, target: &Entity) -> Vec<Entity> {
    let mut reactions = Vec::new();
    let can_react = ecs.read_storage::<super::CanReactFlag>();

    if can_react.get(*target).is_some() {
        reactions.push(*target);
    }

    reactions
}

// placeholder handling
fn process_event(_ecs: &mut World, event: &Event) {
    match event.event_type {
        EventType::Damage { amount } => println!(
            "{:?} took {:?} damage from {:?}",
            event.target, amount, event.source
        ),
    }
}
