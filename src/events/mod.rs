use super::{CardRequest, ParticleRequest};
use rltk::Point;
use specs::prelude::*;
use std::sync::Mutex;

lazy_static! {
    pub static ref STACK: Mutex<Vec<Event>> = Mutex::new(Vec::new());
    pub static ref CARDSTACK: Mutex<Vec<CardRequest>> = Mutex::new(Vec::new());
}

pub enum EventType {
    Damage { amount: i32 },
    ParticleSpawn { request: ParticleRequest },
    ShowCard { request: CardRequest, offset: i32 },
}

pub struct Event {
    pub event_type: EventType,
    pub source: Option<Entity>,
    pub target_tiles: Vec<Point>,
    pub invokes_reaction: bool,
}

pub fn add_event(event_type: EventType, targets: Vec<Point>, invokes_reaction: bool) {
    let mut stack = STACK.lock().expect("Failed to lock STACK");
    let event = Event {
        event_type,
        source: None,
        target_tiles: targets,
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

                    add_card_to_stack(ecs, &entities_hit, &event);

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

fn add_card_to_stack(ecs: &mut World, entities_hit: &Vec<Entity>, event: &Event) {
    let player = ecs.fetch::<Entity>();
    if entities_hit.contains(&*player) {
        let visual_event_data = get_assoc_card_event(event);

        if let Some(visual_event_data) = visual_event_data {
            CARDSTACK
                .lock()
                .expect("Failed to lock CARDSTACK")
                .push(visual_event_data);
        }
    }
}

fn get_assoc_card_event(event: &Event) -> Option<CardRequest> {
    match event.event_type {
        EventType::Damage { .. } => Some(CardRequest {
            name: "Damage".to_string(),
            offset: 0,
        }),
        _ => None,
    }
}

fn process_event(ecs: &mut World, event: Event) {
    let top = CARDSTACK.lock().expect("Failed to lock CARDSTACK").pop();
    let still_alive = {
        let mut count = 0;
        let cards = ecs.read_storage::<super::CardLifetime>();
        for _ in cards.join() {
            count += 1;
        }
        count
    };

    if let Some(top) = top {
        handle_event_types(
            ecs,
            Event {
                event_type: EventType::ShowCard {
                    request: top,
                    offset: still_alive,
                },
                source: None,
                target_tiles: Vec::new(),
                invokes_reaction: false,
            },
        );
    }

    handle_event_types(ecs, event);
}

// placeholder handling
fn handle_event_types(ecs: &mut World, event: Event) {
    match event.event_type {
        EventType::Damage { amount } => {
            for pos in &event.target_tiles {
                add_event(
                    EventType::ParticleSpawn {
                        request: ParticleRequest {
                            position: *pos,
                            color: rltk::RGB::named(rltk::RED),
                            symbol: rltk::to_cp437('█'),
                            lifetime: 600.0,
                        },
                    },
                    Vec::new(),
                    false,
                );
            }

            for ent in get_affected_entities(ecs, &event.target_tiles) {
                println!("{:?} took {:?} damage from {:?}", ent, amount, event.source)
            }
        }
        EventType::ParticleSpawn { request } => {
            let mut builder = ecs.fetch_mut::<super::ParticleBuilder>();
            builder.make_particle(request);
        }
        EventType::ShowCard { request, offset } => {
            let mut builder = ecs.fetch_mut::<super::ParticleBuilder>();
            builder.make_card(request, offset);
        }
    }
}
