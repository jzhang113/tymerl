use super::CardRequest;
use rltk::Point;
use specs::prelude::*;
use std::sync::Mutex;

mod event_type;
mod range_type;

pub use event_type::EventType;
pub use range_type::RangeType;

lazy_static! {
    static ref STACK: Mutex<Vec<Event>> = Mutex::new(Vec::new());
    pub static ref CARDSTACK: Mutex<Vec<CardRequest>> = Mutex::new(Vec::new());
}

struct Event {
    resolver: Box<dyn event_type::EventResolver + Send>,
    name: Option<String>,
    source: Option<Entity>,
    target_tiles: Vec<Point>,
    invokes_reaction: bool,
}

pub fn add_event(event_type: &EventType, range: &RangeType, loc: Point, invokes_reaction: bool) {
    let mut stack = STACK.lock().expect("Failed to lock STACK");
    let event = Event {
        resolver: event_type::get_resolver(event_type),
        name: event_type::get_name(event_type),
        source: None,
        target_tiles: range_type::resolve_range_at(range, loc),
        invokes_reaction,
    };

    stack.push(event);
}

pub fn process_stack(ecs: &mut World) {
    loop {
        let event = STACK.lock().expect("Failed to lock STACK").pop();
        match event {
            None => {
                break;
            }
            Some(event) => {
                if event.target_tiles.is_empty() {
                    // non-targetted events
                    process_event(ecs, event);
                } else {
                    let mut entities_hit = get_affected_entities(ecs, &event.target_tiles);

                    if let Some(card_name) = &event.name {
                        add_card_to_stack(ecs, &entities_hit, card_name);
                    }

                    entities_hit.retain(|ent| entity_can_react(ecs, ent));

                    // check if there are entities that can respond
                    if event.invokes_reaction && !entities_hit.is_empty() {
                        let mut can_act = ecs.write_storage::<super::CanActFlag>();

                        for entity in entities_hit {
                            can_act
                                .insert(entity, super::CanActFlag { is_reaction: true })
                                .expect("Failed to insert CanActFlag");
                        }

                        // put the event back on the stack and return control to the main loop
                        STACK.lock().expect("Failed to lock STACK").push(event);
                        break;
                    } else {
                        // otherwise resolve the event
                        process_event(ecs, event);
                    }
                }
            }
        }
    }
}

fn get_affected_entities(ecs: &mut World, targets: &Vec<Point>) -> Vec<Entity> {
    let mut affected = Vec::new();
    let positions = ecs.read_storage::<crate::Position>();
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

fn add_card_to_stack(ecs: &mut World, entities_hit: &Vec<Entity>, name: &String) {
    let player = ecs.fetch::<Entity>();
    if entities_hit.contains(&*player) {
        let visual_event_data = Some(CardRequest {
            name: name.to_string(),
            offset: 0,
        });

        if let Some(visual_event_data) = visual_event_data {
            CARDSTACK
                .lock()
                .expect("Failed to lock CARDSTACK")
                .push(visual_event_data);
        }
    }
}

fn process_event(ecs: &mut World, event: Event) {
    let top = CARDSTACK.lock().expect("Failed to lock CARDSTACK").pop();
    let still_alive = {
        let mut count = 0;
        let cards = ecs.read_storage::<crate::CardLifetime>();
        for _ in cards.join() {
            count += 1;
        }
        count
    };

    if let Some(top) = top {
        let mut builder = ecs.fetch_mut::<crate::ParticleBuilder>();
        builder.make_card(top, still_alive);
    }

    event
        .resolver
        .resolve(ecs, event.source, event.target_tiles);
}
