
pub mod ecs;

#[cfg(feature = "graphics")]
pub mod graphics;



pub use ecs::plugin::Plugin;

use crate::ecs::scheduler::AddPlugins;
#[cfg(feature = "graphics")]
use crate::graphics::WindowPlugin;


#[derive(Default)]
pub struct DefaultPlugins {
    #[cfg(feature = "graphics")]
    window: WindowPlugin,
}


impl Plugin for DefaultPlugins {
    fn setup(self, mut app: ecs::scheduler::App) -> ecs::scheduler::App {
        #[cfg(feature = "graphics")]
        {
            app = app.add_plugins((
                self.window,
            ))
        }

        app
    }
}

