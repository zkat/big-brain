use bevy::{app::AppExit, prelude::*};
use big_brain::{pickers, prelude::*};

#[test]
fn steps() {
    println!("steps test");
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(BigBrainPlugin)
        .init_resource::<GlobalState>()
        .add_startup_system(setup)
        .add_system_to_stage(CoreStage::First, no_failure_score)
        .add_system(action1)
        .add_system(action2)
        .add_system(exit_action)
        .add_system(failure_action)
        .add_system_to_stage(CoreStage::Last, last)
        .run();
    println!("end");
}

fn setup(mut cmds: Commands) {
    cmds.spawn().insert(
        Thinker::build()
            .picker(pickers::FirstToScore::new(0.5))
            .when(NoFailureScore, Steps::build().step(FailureAction))
            .otherwise(Steps::build().step(Action1).step(Action2).step(ExitAction)),
    );
}

#[derive(Component, Debug, Default)]
struct Action1;
impl ActionBuilder for Action1 {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action)
            .insert(Self)
            .insert(ActionState::Requested);
    }
}

fn action1(mut query: Query<(&Actor, &mut ActionState), With<Action1>>) {
    for (Actor(_actor), mut state) in query.iter_mut() {
        println!("action1 {:?}", state);
        if *state == ActionState::Requested {
            *state = ActionState::Executing;
        }
        if *state == ActionState::Executing {
            *state = ActionState::Success;
        }
    }
}

#[derive(Component, Debug, Default)]
struct Action2;
impl ActionBuilder for Action2 {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action)
            .insert(Self)
            .insert(ActionState::Requested);
    }
}

fn action2(mut query: Query<(&Actor, &mut ActionState), With<Action2>>) {
    for (Actor(_actor), mut state) in query.iter_mut() {
        println!("action2 {:?}", state);
        if *state == ActionState::Requested {
            *state = ActionState::Executing;
        }
        if *state == ActionState::Executing {
            *state = ActionState::Success;
        }
    }
}

#[derive(Component, Debug, Default)]
struct ExitAction;
impl ActionBuilder for ExitAction {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action)
            .insert(Self)
            .insert(ActionState::Requested);
    }
}

fn exit_action(
    mut query: Query<(&Actor, &mut ActionState), With<ExitAction>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for (Actor(_actor), mut state) in query.iter_mut() {
        println!("exit_action {:?}", state);
        if *state == ActionState::Requested {
            *state = ActionState::Executing;
        }
        if *state == ActionState::Executing {
            app_exit_events.send(AppExit);
        }
    }
}

fn last() {
    println!();
}

#[derive(Component, Debug, Default)]
struct FailureAction;
impl ActionBuilder for FailureAction {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action)
            .insert(Self)
            .insert(ActionState::Requested);
    }
}

fn failure_action(
    mut query: Query<(&Actor, &mut ActionState), With<FailureAction>>,
    mut global_state: ResMut<GlobalState>,
) {
    for (Actor(_actor), mut state) in query.iter_mut() {
        println!("failure_action {:?}", state);
        if *state == ActionState::Requested {
            *state = ActionState::Executing;
        }
        if *state == ActionState::Executing {
            global_state.failure = true;
            *state = ActionState::Failure;
        }
    }
}

#[derive(Default)]
struct GlobalState {
    failure: bool,
}

#[derive(Clone, Component, Debug)]
struct NoFailureScore;

fn no_failure_score(
    mut query: Query<(&NoFailureScore, &mut Score)>,
    global_state: Res<GlobalState>,
) {
    for (_, mut score) in query.iter_mut() {
        score.set(if global_state.failure { 0.0 } else { 1.0 });
    }
}
