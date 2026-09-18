use crate::scheduler::App;




pub trait Plugin {
    fn setup(self, app: App) -> App;
}

