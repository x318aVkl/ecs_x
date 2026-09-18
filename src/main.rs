use ecs::{commands::{Commands, SpawnEntity}, plugin::Plugin, query::{Query, QueryIter, With}, scheduler::{AddPlugins, AddSystems, App, Enter, Exit, Startup, Update}, state::State, system::{Res, ResMut}};



struct Moving {
    velocity: f32,
}
struct Still;


struct MyPlugin;

impl Plugin for MyPlugin {
    fn setup(self, app: App) -> App {
        app
            .add_systems::<Enter<Moving>>((enter_moving,))
            .add_systems::<Exit<Moving>>((exit_moving,))
            .add_systems::<Enter<Still>>((enter_still,))
            .add_systems::<Exit<Still>>((exit_still,))
    }
}


fn main() {

    App::new()
        .add_plugins((MyPlugin,))
        .add_systems::<Startup>((foo,))
        .add_systems::<Update>((bar, gogo))
        .run()
        .unwrap();

}


fn foo(
    mut commands: Commands,
) {
    commands.insert_resource(0_u32);
    commands.insert_resource(20_i32);

    for i in 0..1_000 {
        commands.spawn((i as i32, 1_u32,));
    }
}

fn bar(
    mut value: ResMut<u32>,
    q: Query<(&mut i32, &u32), (With<u32>,)>,
    mut commands: Commands,
) {

    let mut k = 0;
    for (e, (mut x, _y)) in q.iter() {
        *x = 3;
        if k < 10 {
            commands.despawn(e);
        }
        k += 1;
    }

    *value += 1;
    println!("{:?}", *value);

    if *value == 20 {
        commands.change_state(Moving {
            velocity: 0.5,
        });
    }

    if *value == 30 {
        commands.change_state(Still);
    }

    if *value > 60 {
        commands.exit(0);
    }
}

fn gogo(
    res: Res<u32>,
    moving: Option<State<Moving>>,
    mut commands: Commands,
) {

    if let Some(moving) = moving {
        println!("currently moving {:?}", moving.velocity);
    } else {
        println!("not moving")
    }
}


fn enter_moving(

) {
    println!("Entered moving state");
}


fn exit_moving(

) {
    println!("Exit moving state");
}

fn enter_still(

) {
    println!("Entered still state");
}


fn exit_still(

) {
    println!("Exit still state");
}

