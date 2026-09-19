
use ecs_x::{DefaultPlugins, ecs::{commands::{Commands, SpawnEntity}, event::{EventPlugin, EventReader, EventWriter}, plugin::Plugin, query::{Query, QueryIter, With}, scheduler::{AddPlugins, AddSystems, App, Enter, Exit, Startup, Update}, state::State, system::{Res, ResMut}}, graphics::{Window, WindowPlugin, window::{Down, Pointer, PointerMoved, PointerPosition, PrimaryWindow}}};



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

struct MyEvent {
    value: i32,
}

fn main() {

    App::new()
        .add_plugins((
            DefaultPlugins::default(),
            MyPlugin,
            EventPlugin::<MyEvent>::default(),
        ))
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

    println!("done with startup stuff");
}

fn bar(
    mut value: ResMut<u32>,
    q: Query<(&mut i32, &u32), (With<u32>,)>,
    mut commands: Commands,
    mut writer: ResMut<EventWriter<MyEvent>>,
    windows: Query<(&Window,), (With<PrimaryWindow>,)>,
) {

    let mut k = 0;
    for (e, (mut x, _y)) in q.iter() {
        *x += 3;
        if k < 10 {
            commands.despawn(e);
        }
        k += 1;
    }

    *value += 1;
    println!("{} {}", k, *value);

    if *value == 4 {
        writer.write(MyEvent { value: 0 });
    }

    if *value == 20 {
        commands.change_state(Moving {
            velocity: 0.5,
        });
    }

    if *value == 30 {
        commands.change_state(Still);
    }

    if *value == 2000 {
        if let Some((window, _)) = windows.iter().nth(0) {
            println!("despawining window");
            commands.despawn(window);
        }
    }

}

fn gogo(
    res: Res<u32>,
    moving: Option<State<Moving>>,
    mut commands: Commands,
    reader: Res<EventReader<MyEvent>>,
    pointer_reader: Res<EventReader<Pointer<Down>>>,
    pmove_reader: Res<EventReader<PointerMoved>>,
    position: Res<PointerPosition>,
) {

    for event in reader.read() {
        println!("got an event! {}", event.value);
    }

    for event in pointer_reader.read() {
        println!("got a pointer event! {:?}", event.button);
    }

    for event in pmove_reader.read() {
        println!("moved pointer: {} {}", event.dx, event.dy);
        println!("pointer position: {} {}", position.x, position.y);
    }

    if let Some(moving) = moving {
        println!("currently moving {:?}", moving.velocity);
    } else {
        //println!("not moving")
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

