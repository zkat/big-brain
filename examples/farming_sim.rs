//! Simple example of a utility ai farming agent

use bevy::{log::LogPlugin, prelude::*};
use bevy_editor_pls::EditorPlugin;
use bevy_scene_hook::{HookPlugin, HookedSceneBundle, SceneHook};
use big_brain::prelude::*;
use big_brain_derive::ActionBuilder;

const DEFAULT_COLOR: Color = Color::BLACK;
const SLEEP_COLOR: Color = Color::RED;
const FARM_COLOR: Color = Color::BLUE;
const MAX_DISTANCE: f32 = 0.1;
const MAX_INVENTORY_ITEMS: f32 = 20.0;

#[derive(Component, Debug, Clone)]
pub struct Field;

#[derive(Component, Debug, Clone)]
pub struct Market;

#[derive(Component, Debug, Clone)]
pub struct House;

#[derive(Component, Reflect)]
pub struct Inventory {
    pub money: u32,
    pub items: f32,
}

#[derive(Component)]
pub struct MoneyText;

#[derive(Component)]
pub struct FatigueText;

#[derive(Component)]
pub struct InventoryText;

// ================================================================================
//  Sleepiness 😴
// ================================================================================
#[derive(Component, Debug, Reflect)]
pub struct Fatigue {
    pub is_sleeping: bool,
    pub per_second: f32,
    pub level: f32,
}

pub fn fatigue_system(time: Res<Time>, mut fatigues: Query<&mut Fatigue>) {
    for mut fatigue in &mut fatigues {
        fatigue.level += fatigue.per_second * time.delta_seconds();
        if fatigue.level >= 100.0 {
            fatigue.level = 100.0;
        }
        trace!("Tiredness: {}", fatigue.level);
    }
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct Sleep {
    until: f32,
    per_second: f32,
}

fn sleep_action_system(
    time: Res<Time>,
    mut fatigues: Query<(&mut Fatigue, &Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&Actor, &mut ActionState, &Sleep, &ActionSpan)>,
) {
    for (Actor(actor), mut state, sleep, span) in &mut query {
        let _guard = span.span().enter();

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
                        debug!("Woke up well-rested!");
                        materials.get_mut(material).unwrap().base_color = DEFAULT_COLOR;
                        fatigue.is_sleeping = false;
                        *state = ActionState::Success;
                    }
                }
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

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct FatigueScorer;

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

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct Farm {
    pub until: f32,
    pub per_second: f32,
}

fn farm_action_system(
    time: Res<Time>,
    mut actors: Query<(&mut Inventory, &Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct WorkNeedScorer;

pub fn work_need_scorer_system(
    actors: Query<&Inventory>,
    mut query: Query<(&Actor, &mut Score), With<WorkNeedScorer>>,
) {
    for (Actor(actor), mut score) in &mut query {
        if let Ok(inventory) = actors.get(*actor) {
            if inventory.items >= MAX_INVENTORY_ITEMS {
                score.set(0.0);
            } else {
                score.set(0.6);
            }
        }
    }
}

// ================================================================================
//  Selling 💰
// ================================================================================

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct Sell;

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

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct SellNeedScorer;

pub fn sell_need_scorer_system(
    actors: Query<&Inventory>,
    mut query: Query<(&Actor, &mut Score), With<SellNeedScorer>>,
) {
    for (Actor(actor), mut score) in &mut query {
        if let Ok(inventory) = actors.get(*actor) {
            if inventory.items >= MAX_INVENTORY_ITEMS {
                score.set(0.6);
            } else {
                score.set(0.0);
            }
        }
    }
}

// ================================================================================
//  Movement 🚶
// ================================================================================

#[derive(Clone, Component, Debug)]
pub struct MoveToNearest<T: Component + std::fmt::Debug + Clone> {
    _marker: std::marker::PhantomData<T>,
    speed: f32,
}

impl<T> ActionBuilder for MoveToNearest<T>
where
    T: Component + std::fmt::Debug + Clone,
{
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action).insert(MoveToNearest::<T>::clone(self));
    }
}

pub fn move_to_nearest_system<T: Component + std::fmt::Debug + Clone>(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<T>>,
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
                let goal_transform = query
                    .iter_mut()
                    .map(|t| (t.translation, t))
                    .min_by(|(a, _), (b, _)| {
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

fn update_ui(
    actor_query: Query<(&Inventory, &Fatigue)>,
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
            text.sections[0].value = format!("Money: {}", inventory.money as u32);
        }

        for mut text in &mut fatigue_query {
            text.sections[0].value = format!("Fatigue: {}", fatigue.level as u32);
        }

        for mut text in &mut inventory_query {
            text.sections[0].value = format!("Inventory: {}", inventory.items as u32);
        }
    }
}

fn init_entities(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(6.0, 6.0, 4.0)
            .looking_at(Vec3::new(0.0, -1.0, 0.0), Vec3::Y),
        ..default()
    },));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });

    commands.spawn((
        Name::new("Light"),
        SpotLightBundle {
            spot_light: SpotLight {
                shadows_enabled: true,
                intensity: 5_000.0,
                range: 100.0,
                ..default()
            },
            transform: Transform::from_xyz(2.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
    ));

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

    let move_and_sleep = Steps::build()
        .label("MoveAndSleep")
        .step(MoveToNearest::<House> {
            speed: 1.0,
            _marker: std::marker::PhantomData,
        })
        .step(Sleep {
            until: 10.0,
            per_second: 10.0,
        });

    let move_and_farm = Steps::build()
        .label("MoveAndFarm")
        .step(MoveToNearest::<Field> {
            speed: 1.0,
            _marker: std::marker::PhantomData,
        })
        .step(Farm {
            until: 10.0,
            per_second: 10.0,
        });

    let move_and_sell = Steps::build()
        .label("MoveAndSell")
        .step(MoveToNearest::<Market> {
            speed: 1.0,
            _marker: std::marker::PhantomData,
        })
        .step(Sell);

    commands.spawn((
        Name::new("Farmer"),
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                depth: 0.3,
                radius: 0.1,
                ..default()
            })),
            material: materials.add(DEFAULT_COLOR.into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Fatigue {
            is_sleeping: false,
            per_second: 2.0,
            level: 0.0,
        },
        Inventory {
            money: 0,
            items: 0.0,
        },
        Thinker::build()
            .label("My Thinker")
            .picker(FirstToScore::new(0.6))
            .when(FatigueScorer, move_and_sleep)
            .when(WorkNeedScorer, move_and_farm)
            .when(SellNeedScorer, move_and_sell),
    ));

    let font_size = 40.0;

    // scoreboard
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
            builder.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size,
                        ..default()
                    },
                ),
                MoneyText,
            ));

            builder.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size,
                        ..default()
                    },
                ),
                FatigueText,
            ));

            builder.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size,
                        ..default()
                    },
                ),
                InventoryText,
            ));
        });
}

fn main() {
    App::new()
        // .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::WARN,
            // Use `RUST_LOG=big_brain=trace,farming_sim=trace cargo run --example
            // farming_sim --features=trace` to see extra tracing output.
            // filter: "big_brain=debug,farming_sim=trace".to_string(),
            ..default()
        }))
        .register_type::<Fatigue>()
        .register_type::<Inventory>()
        .add_plugins(EditorPlugin::default())
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
