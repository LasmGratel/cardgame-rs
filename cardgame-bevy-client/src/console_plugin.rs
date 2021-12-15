use bevy::{
    ecs::{archetype::Archetypes, component::Components, entity::Entities},
    prelude::*,
    reflect::TypeRegistry,
    tasks::AsyncComputeTaskPool,
};
use crossbeam::channel::{bounded, Receiver};
use std::io::{self, BufRead, Write};
use bevy::ecs::schedule::ShouldRun;

pub struct InputEvent(pub String);

fn parse_input(
    time: Res<Time>,
    line_channel: Res<Receiver<String>>,
    mut events: EventWriter<InputEvent>,
) {
    if let Ok(line) = line_channel.try_recv() {
        events.send(InputEvent(line));
        io::stdout().flush().unwrap();
    }
}

fn spawn_io_thread(mut commands: Commands, thread_pool: Res<AsyncComputeTaskPool>) {
    println!("Bevy Console Debugger.  Type 'help' for list of commands.");
    print!(">>> ");
    io::stdout().flush().unwrap();

    let (tx, rx) = bounded(2);
    let task = thread_pool.spawn(async move {
        let stdin = io::stdin();
        loop {
            let line = stdin.lock().lines().next().unwrap().unwrap();
            tx.send(line)
                .expect("error sending user input to other thread");
        }
    });
    task.detach();
    commands.insert_resource(rx);
}

pub struct ConsoleDebugPlugin;
impl Plugin for ConsoleDebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_event::<InputEvent>()
            .add_startup_system(spawn_io_thread.system())
            .add_system(parse_input.system())
        ;
    }
}