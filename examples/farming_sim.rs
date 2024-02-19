//! An intermediate example of a utility AI agent for a farmer
//!
//! The farmer agent will:
//! - Get tired over time, indicated by their [Fatigue] component
//! - When tired, find the house and sleep to reduce fatigue
//! - When not tired, find the farm field and harvest items over time
//! - When inventory is full, find the market and sell items for money

use bevy::{log::LogPlugin, prelude::*};
use bevy_scene_hook::{HookPlugin, HookedSceneBundle, SceneHook};
use big_brain::prelude::*;
use big_brain_derive::ActionBuilder;

const DEFAULT_COLOR: Color = Color::BLACK;
const SLEEP_COLOR: Color = Color::RED;
const FARM_COLOR: Color = Color::BLUE;
const MAX_DISTANCE: f32 = 0.1;
const MAX_INVENTORY_ITEMS: f32 = 20.0;
const WORK_NEED_SCORE: f32 = 0.6;
const SELL_NEED_SCORE: f32 = 0.6;
const MOVEMENT_SPEED: f32 = 1.5;

/// A marker for our spawned gltf indicating the farm's field location.
#[derive(Component, Debug, Clone)]
pub struct Field;

/// A marker for our spawned gltf indicating the market's location.
#[derive(Component, Debug, Clone)]
pub struct Market;

/// A marker for our spawned gltf indicating the house's location.
#[derive(Component, Debug, Clone)]
pub struct House;

/// The farmer's inventory.
#[derive(Component, Reflect)]
pub struct Inventory {
    /// How much money this entity has.
    pub money: u32,
    /// How many items the entity has.
    // We use a float here to simplify the math in the farming action.
    pub items: f32,
}

/// A marker for our money UI text.
#[derive(Component)]
pub struct MoneyText;

/// A marker for our fatigue UI text.
#[derive(Component)]
pub struct FatigueText;

/// A marker for our inventory UI text.
#[derive(Component)]
pub struct InventoryText;

// ================================================================================
//  Sleepiness 😴
// ================================================================================

// This is not an AI component, but a standard Bevy component that increases an
// entity's fatigue over time. The AI will interact with this component later.
#[derive(Component, Debug, Reflect)]
pub struct Fatigue {
    /// A boolean indicating whether the entity is currently sleeping.
    pub is_sleeping: bool,
    /// The rate at which the fatigue level increases per second.
    pub per_second: f32,
    /// The current fatigue level of the entity.
    pub level: f32,
}

/// Increases an entity's fatigue over time
pub fn fatigue_system(time: Res<Time>, mut fatigues: Query<&mut Fatigue>) {
    for mut fatigue in &mut fatigues {
        fatigue.level += fatigue.per_second * time.delta_seconds();
        if fatigue.level >= 100.0 {
            fatigue.level = 100.0;
        }
        trace!("Tiredness: {}", fatigue.level);
    }
}

// The second step is to define an action. What can the AI do, and how does it
// do it? This is the first bit involving Big Brain itself, and there's a few
// pieces you need:
//
// 1. An Action Component. This is just a plain Component we will query
//    against later.
// 2. An ActionBuilder. This is anything that implements the ActionBuilder
//    trait.
// 3. A System that will run Action code.
//
// These actions will be spawned and queued by the game engine when their
// conditions trigger (we'll configure what these are later).
//
// In most cases, the ActionBuilder just attaches the Action component to the
// actor entity. In this case, you can use the derive macro `ActionBuilder`
// to make your Action Component implement the ActionBuilder trait.
// You need your type to implement Clone and Debug (necessary for ActionBuilder)
#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct Sleep {
    /// The fatigue level at which the entity will stop sleeping.
    until: f32,
    /// The rate at which the fatigue level decreases while sleeping.
    per_second: f32,
}

// This system manages the sleeping action of entities. It reduces the fatigue
// level of the entity as it sleeps and updates the entity's state based on
// the Sleep component's parameters.
fn sleep_action_system(
    time: Res<Time>,
    mut fatigues: Query<(&mut Fatigue, &Handle<StandardMaterial>)>,
    // Resource used to modify the appearance of the farmer.
    mut materials: ResMut<Assets<StandardMaterial>>,
    // We execute actions by querying for their associated Action Component
    // (Sleep in this case). You'll always need both Actor and ActionState.
    mut query: Query<(&Actor, &mut ActionState, &Sleep, &ActionSpan)>,
) {
    for (Actor(actor), mut state, sleep, span) in &mut query {
        // This sets up the tracing scope. Any `debug` calls here will be
        // spanned together in the output.
        let _guard = span.span().enter();

        // Use the sleep_action's actor to look up the corresponding Fatigue Component.
        if let Ok((mut fatigue, material)) = fatigues.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    debug!("Time to sleep!");
                    fatigue.is_sleeping = true;
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    trace!("Sleeping...");
                    fatigue.level -= sleep.per_second * time.delta_seconds();
                    materials.get_mut(material).unwrap().base_color = SLEEP_COLOR;

                    if fatigue.level <= sleep.until {
                        // To "finish" an action, we set its state to Success or
                        // Failure.
                        debug!("Woke up well-rested!");
                        materials.get_mut(material).unwrap().base_color = DEFAULT_COLOR;
                        fatigue.is_sleeping = false;
                        *state = ActionState::Success;
                    }
                }
                // All Actions should make sure to handle cancellations!
                ActionState::Cancelled => {
                    debug!("Sleep was interrupted. Still tired.");
                    materials.get_mut(material).unwrap().base_color = DEFAULT_COLOR;
                    fatigue.is_sleeping = false;
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

/// This component serves as a scorer for evaluating the entity's need to sleep based on its fatigue level.
#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct FatigueScorer;

// This system calculates a score based on the entity's fatigue level. The higher the fatigue, the higher
// the score, indicating a greater need for the entity to sleep.
pub fn fatigue_scorer_system(
    mut last_score: Local<Option<f32>>,
    fatigues: Query<&Fatigue>,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<FatigueScorer>>,
) {
    for (Actor(actor), mut score, span) in &mut query {
        if let Ok(fatigue) = fatigues.get(*actor) {
            let new_score = fatigue.level / 100.0;

            if fatigue.is_sleeping {
                let _score = last_score.get_or_insert(new_score);

                score.set(*_score);
            } else {
                last_score.take();
                score.set(new_score);
                if fatigue.level >= 80.0 {
                    span.span().in_scope(|| {
                        debug!("Fatigue above threshold! Score: {}", fatigue.level / 100.0)
                    });
                }
            }
        }
    }
}

// ================================================================================
//  Farming 🚜
// ================================================================================

/// Represents the farming action. When the farmer decides to farm, this component
/// is used to track and manage the farming process.
#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct Farm {
    /// The threshold at which the farmer stops farming (e.g., when the inventory is full).
    pub until: f32,
    /// The rate at which items are added to the inventory per second while farming.
    pub per_second: f32,
}

// The system that executes the farming action. It updates the inventory based on the
// Farm component's parameters and changes the entity's appearance to indicate the farming action.
fn farm_action_system(
    time: Res<Time>,
    mut actors: Query<(&mut Inventory, &Handle<StandardMaterial>)>,
    // Resource used to modify the appearance of the farmer.
    mut materials: ResMut<Assets<StandardMaterial>>,
    // Query to manage the state of the farming action.
    mut query: Query<(&Actor, &mut ActionState, &Farm, &ActionSpan)>,
) {
    for (Actor(actor), mut state, farm, span) in &mut query {
        let _guard = span.span().enter();

        if let Ok((mut inventory, material)) = actors.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    debug!("Time to farm!");
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    trace!("Farming...");
                    inventory.items += farm.per_second * time.delta_seconds();
                    materials.get_mut(material).unwrap().base_color = FARM_COLOR;

                    if inventory.items >= MAX_INVENTORY_ITEMS {
                        debug!("Inventory full!");
                        materials.get_mut(material).unwrap().base_color = DEFAULT_COLOR;
                        *state = ActionState::Success;
                    }
                }
                ActionState::Cancelled => {
                    debug!("Farming was interrupted. Still need to work.");
                    materials.get_mut(material).unwrap().base_color = DEFAULT_COLOR;
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

/// This component serves as a scorer for evaluating the entity's need to farm based on its inventory level.
#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct WorkNeedScorer;

// This scorer returns a score of the default work need score if the entity's inventory is not full and 0.0 otherwise.
pub fn work_need_scorer_system(
    actors: Query<&Inventory>,
    mut query: Query<(&Actor, &mut Score), With<WorkNeedScorer>>,
) {
    for (Actor(actor), mut score) in &mut query {
        if let Ok(inventory) = actors.get(*actor) {
            if inventory.items >= MAX_INVENTORY_ITEMS {
                score.set(0.0);
            } else {
                score.set(WORK_NEED_SCORE);
            }
        }
    }
}

// ================================================================================
//  Selling 💰
// ================================================================================

/// Represents the selling action. When the farmer decides to sell, this component
/// is used to track and manage the selling process.
#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct Sell;

/// The system that executes the selling action. It updates the inventory based on the
/// Sell component's parameters and changes the entity's appearance to indicate the selling action.
fn sell_action_system(
    mut actors: Query<&mut Inventory>,
    mut query: Query<(&Actor, &mut ActionState, &Sell, &ActionSpan)>,
) {
    for (Actor(actor), mut state, _sell, span) in &mut query {
        let _guard = span.span().enter();

        if let Ok(mut inventory) = actors.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    debug!("Time to sell!");
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    trace!("Selling...");
                    inventory.money += inventory.items as u32;
                    inventory.items = 0.0;

                    debug!("Sold! Money: {}", inventory.money);

                    // Notice we immediately set the state to Success. This is because
                    // we treat selling as instantaneous.
                    *state = ActionState::Success;
                }
                ActionState::Cancelled => {
                    debug!("Selling was interrupted. Still need to work.");
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

// This component serves as a scorer for evaluating the entity's need to sell based on its inventory level.
#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct SellNeedScorer;

// This scorer returns a score of the default sell need score if the entity's inventory is full and 0.0 otherwise.
pub fn sell_need_scorer_system(
    actors: Query<&Inventory>,
    mut query: Query<(&Actor, &mut Score), With<SellNeedScorer>>,
) {
    for (Actor(actor), mut score) in &mut query {
        if let Ok(inventory) = actors.get(*actor) {
            if inventory.items >= MAX_INVENTORY_ITEMS {
                score.set(SELL_NEED_SCORE);
            } else {
                score.set(0.0);
            }
        }
    }
}

// ================================================================================
//  Movement 🚶
// ================================================================================

// This is a component that will be attached to the actor entity when it is
// moving to a location. It's not an AI component, but a standard Bevy component
#[derive(Debug, Clone, Component, ActionBuilder)]
#[action_label = "MyGenericLabel"]
pub struct MoveToNearest<T: Component + std::fmt::Debug + Clone> {
    // We use a PhantomData to store the type of the component we're moving to.
    _marker: std::marker::PhantomData<T>,
    speed: f32,
}

impl<T: Component + std::fmt::Debug + Clone> MoveToNearest<T> {
    pub fn new(speed: f32) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            speed,
        }
    }
}

// This system manages the movement of entities. It moves the entity towards the
// nearest entity with the specified component and updates the entity's state
// based on the MoveToNearest component's parameters.
pub fn move_to_nearest_system<T: Component + std::fmt::Debug + Clone>(
    time: Res<Time>,
    // This will be generic over 'T', so we can look up any marker component we want.
    mut query: Query<&mut Transform, With<T>>,
    // We filter on HasThinker since otherwise we'd be querying for every
    // entity in the world with a transform!
    mut thinkers: Query<&mut Transform, (With<HasThinker>, Without<T>)>,
    mut action_query: Query<(&Actor, &mut ActionState, &MoveToNearest<T>, &ActionSpan)>,
) {
    for (actor, mut action_state, move_to, span) in &mut action_query {
        let _guard = span.span().enter();

        match *action_state {
            ActionState::Requested => {
                debug!("Let's go find a {:?}", std::any::type_name::<T>());

                *action_state = ActionState::Executing;
            }
            ActionState::Executing => {
                let mut actor_transform = thinkers.get_mut(actor.0).unwrap();
                // The goal is the nearest entity with the specified component.
                let goal_transform = query
                    .iter_mut()
                    .map(|t| (t.translation, t))
                    .min_by(|(a, _), (b, _)| {
                        // We need partial_cmp here because f32 doesn't implement Ord.
                        let delta_a = *a - actor_transform.translation;
                        let delta_b = *b - actor_transform.translation;
                        delta_a.length().partial_cmp(&delta_b.length()).unwrap()
                    })
                    .unwrap()
                    .1;
                let delta = goal_transform.translation - actor_transform.translation;
                let distance = delta.xz().length();

                trace!("Distance: {}", distance);

                if distance > MAX_DISTANCE {
                    trace!("Stepping closer.");

                    let step_size = time.delta_seconds() * move_to.speed;
                    let step = delta.normalize() * step_size.min(distance);

                    // We only care about moving in the XZ plane.
                    actor_transform.translation.x += step.x;
                    actor_transform.translation.z += step.z;
                } else {
                    debug!("We got there!");

                    *action_state = ActionState::Success;
                }
            }
            ActionState::Cancelled => {
                *action_state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

// ================================================================================
//  UI
// ================================================================================

// This system updates the UI to reflect the state of the farmer.
#[allow(clippy::type_complexity)]
fn update_ui(
    actor_query: Query<(&Inventory, &Fatigue)>,
    // Our queries must be "disjoint", so we use the `Without` component to
    // ensure that we do not query for the same entity twice.
    mut money_query: Query<
        &mut Text,
        (
            With<MoneyText>,
            Without<FatigueText>,
            Without<InventoryText>,
        ),
    >,
    mut fatigue_query: Query<
        &mut Text,
        (
            With<FatigueText>,
            Without<InventoryText>,
            Without<MoneyText>,
        ),
    >,
    mut inventory_query: Query<
        &mut Text,
        (
            With<InventoryText>,
            Without<FatigueText>,
            Without<MoneyText>,
        ),
    >,
) {
    for (inventory, fatigue) in &mut actor_query.iter() {
        for mut text in &mut money_query {
            text.sections[0].value = format!("Money: {}", inventory.money);
        }

        for mut text in &mut fatigue_query {
            text.sections[0].value = format!("Fatigue: {}", fatigue.level as u32);
        }

        for mut text in &mut inventory_query {
            text.sections[0].value = format!("Inventory: {}", inventory.items as u32);
        }
    }
}

// Now that we have all that defined, it's time to add a Thinker to an entity and setup our environment.
fn init_entities(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(6.0, 6.0, 4.0)
            .looking_at(Vec3::new(0.0, -1.0, 0.0), Vec3::Y),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 700.0,
    });

    commands.spawn((
        Name::new("Light"),
        SpotLightBundle {
            spot_light: SpotLight {
                shadows_enabled: true,
                intensity: 500_000.0,
                range: 100.0,
                ..default()
            },
            transform: Transform::from_xyz(2.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
    ));

    // We use a HookedSceneBundle to attach a SceneHook to the entity. This hook
    // will be called when the entity is spawned, and will allow us to insert
    // additional components into the spawned entities.
    commands.spawn((
        Name::new("Town"),
        HookedSceneBundle {
            scene: SceneBundle {
                scene: asset_server.load("models/town.glb#Scene0"),
                ..default()
            },
            hook: SceneHook::new(|entity, cmds| {
                match entity.get::<Name>().map(|t| t.as_str()) {
                    Some("Farm_Marker") => cmds.insert(Field),
                    Some("Market_Marker") => cmds.insert(Market),
                    Some("House_Marker") => cmds.insert(House),
                    _ => cmds,
                };
            }),
        },
    ));

    // We'll use `Steps` to execute a sequence of actions.
    // First, we'll move to the nearest house and sleep, then we'll move to the
    // nearest field and farm, then we'll move to the nearest market and sell.
    // See the `sequence.rs` example for more details.

    let move_and_sleep = Steps::build()
        .label("MoveAndSleep")
        .step(MoveToNearest::<House>::new(MOVEMENT_SPEED))
        .step(Sleep {
            until: 10.0,
            per_second: 15.0,
        });

    let move_and_farm = Steps::build()
        .label("MoveAndFarm")
        .step(MoveToNearest::<Field>::new(MOVEMENT_SPEED))
        .step(Farm {
            until: 10.0,
            per_second: 10.0,
        });

    let move_and_sell = Steps::build()
        .label("MoveAndSell")
        .step(MoveToNearest::<Market>::new(MOVEMENT_SPEED))
        .step(Sell);

    commands.spawn((
        Name::new("Farmer"),
        PbrBundle {
            mesh: meshes.add(Mesh::from(Capsule3d {
                half_length: 0.15,
                radius: 0.1,
                ..default()
            })),
            material: materials.add(DEFAULT_COLOR),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Fatigue {
            is_sleeping: false,
            per_second: 4.0,
            level: 0.0,
        },
        Inventory {
            money: 0,
            items: 0.0,
        },
        Thinker::build()
            .label("My Thinker")
            // Selects the action with the highest score that is above the threshold.
            .picker(FirstToScore::new(0.6))
            .when(FatigueScorer, move_and_sleep)
            .when(WorkNeedScorer, move_and_farm)
            .when(SellNeedScorer, move_and_sell),
    ));

    let style = TextStyle {
        font_size: 40.0,
        ..default()
    };

    // Our scoreboard.
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::End,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|builder| {
            builder.spawn((TextBundle::from_section("", style.clone()), MoneyText));
            builder.spawn((TextBundle::from_section("", style.clone()), FatigueText));
            builder.spawn((TextBundle::from_section("", style.clone()), InventoryText));
        });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::WARN,
            // Use `RUST_LOG=big_brain=trace,farming_sim=trace cargo run --example
            // farming_sim --features=trace` to see extra tracing output.
            filter: "big_brain=debug,farming_sim=debug".to_string(),
            update_subscriber: None,
        }))
        .add_plugins(HookPlugin)
        .add_plugins(BigBrainPlugin::new(PreUpdate))
        .add_systems(Startup, init_entities)
        .add_systems(Update, (fatigue_system, update_ui))
        .add_systems(
            PreUpdate,
            (
                (
                    sleep_action_system,
                    farm_action_system,
                    sell_action_system,
                    move_to_nearest_system::<House>,
                    move_to_nearest_system::<Field>,
                    move_to_nearest_system::<Market>,
                )
                    .in_set(BigBrainSet::Actions),
                (
                    fatigue_scorer_system,
                    work_need_scorer_system,
                    sell_need_scorer_system,
                )
                    .in_set(BigBrainSet::Scorers),
            ),
        )
        .run();
}
