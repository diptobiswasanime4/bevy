use crate::{
    default_stage,
    plugin::{load_plugin, AppPlugin},
    schedule_plan::SchedulePlan,
    App, Events,
};

use legion::prelude::{Resources, Runnable, Schedulable, Universe, World};

static APP_MISSING_MESSAGE: &str = "This AppBuilder no longer has an App. Check to see if you already called run(). A call to app_builder.run() consumes the AppBuilder's App.";

pub struct AppBuilder {
    app: Option<App>,
    schedule_plan: SchedulePlan,
    startup_schedule_plan: SchedulePlan,
}

impl Default for AppBuilder {
    fn default() -> Self {
        let mut app_builder = AppBuilder {
            app: Some(App::default()),
            schedule_plan: SchedulePlan::default(),
            startup_schedule_plan: SchedulePlan::default(),
        };

        app_builder.add_default_stages();
        app_builder
    }
}

impl AppBuilder {
    pub fn empty() -> AppBuilder {
        AppBuilder {
            app: Some(App::default()),
            schedule_plan: SchedulePlan::default(),
            startup_schedule_plan: SchedulePlan::default(),
        }        
    }

    pub fn app(&self) -> &App {
        self.app.as_ref().expect(APP_MISSING_MESSAGE)
    }

    pub fn app_mut(&mut self) -> &mut App {
        self.app.as_mut().expect(APP_MISSING_MESSAGE)
    }

    pub fn world(&self) -> &World {
        &self.app().world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.app_mut().world
    }

    pub fn universe(&self) -> &Universe {
        &self.app().universe
    }

    pub fn universe_mut(&mut self) -> &mut Universe {
        &mut self.app_mut().universe
    }

    pub fn resources(&self) -> &Resources {
        &self.app().resources
    }

    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.app_mut().resources
    }

    pub fn build_and_run_startup_schedule(&mut self) -> &mut Self {
        let mut startup_schedule = self.startup_schedule_plan.build();
        let app = self.app_mut();
        startup_schedule.execute(&mut app.world, &mut app.resources);
        self
    }

    pub fn build_schedule(&mut self) -> &mut Self {
        self.app_mut().schedule = Some(self.schedule_plan.build());
        self
    }

    pub fn run(&mut self) {
        self.build_schedule();
        self.build_and_run_startup_schedule();
        self.app.take().unwrap().run();
    }

    pub fn set_world(&mut self, world: World) -> &mut Self {
        self.app_mut().world = world;
        self
    }

    pub fn setup(&mut self, setup: impl Fn(&mut World, &mut Resources)) -> &mut Self {
        let app = self.app_mut();
        setup(&mut app.world, &mut app.resources);
        self
    }

    pub fn add_stage(&mut self, stage_name: &str) -> &mut Self {
        self.schedule_plan.add_stage(stage_name);
        self
    }

    pub fn add_stage_after(&mut self, target: &str, stage_name: &str) -> &mut Self {
        self.schedule_plan.add_stage_after(target, stage_name);
        self
    }

    pub fn add_stage_before(&mut self, target: &str, stage_name: &str) -> &mut Self {
        self.schedule_plan.add_stage_before(target, stage_name);
        self
    }

    pub fn add_startup_stage(&mut self, stage_name: &str) -> &mut Self {
        self.startup_schedule_plan.add_stage(stage_name);
        self
    }

    pub fn add_system(&mut self, system: Box<dyn Schedulable>) -> &mut Self {
        self.add_system_to_stage(default_stage::UPDATE, system)
    }

    pub fn add_startup_system_to_stage(
        &mut self,
        stage_name: &str,
        system: Box<dyn Schedulable>,
    ) -> &mut Self {
        self.startup_schedule_plan
            .add_system_to_stage(stage_name, system);
        self
    }

    pub fn add_startup_system(&mut self, system: Box<dyn Schedulable>) -> &mut Self {
        self.startup_schedule_plan
            .add_system_to_stage(default_stage::STARTUP, system);
        self
    }

    pub fn add_default_stages(&mut self) -> &mut Self {
        self.add_startup_stage(default_stage::STARTUP)
            .add_stage(default_stage::FIRST)
            .add_stage(default_stage::EVENT_UPDATE)
            .add_stage(default_stage::UPDATE)
            .add_stage(default_stage::LAST)
    }

    pub fn build_system<F>(&mut self, build: F) -> &mut Self
    where
        F: Fn(&mut Resources) -> Box<dyn Schedulable>,
    {
        let system = build(self.resources_mut());
        self.add_system(system)
    }

    pub fn add_system_to_stage(
        &mut self,
        stage_name: &str,
        system: Box<dyn Schedulable>,
    ) -> &mut Self {
        self.schedule_plan.add_system_to_stage(stage_name, system);
        self
    }

    pub fn add_thread_local_to_stage(
        &mut self,
        stage_name: &str,
        system: Box<dyn Runnable>,
    ) -> &mut Self {
        self.schedule_plan
            .add_thread_local_to_stage(stage_name, system);
        self
    }

    pub fn add_thread_local_fn_to_stage(
        &mut self,
        stage_name: &str,
        f: impl FnMut(&mut World, &mut Resources) + 'static,
    ) -> &mut Self {
        self.schedule_plan
            .add_thread_local_fn_to_stage(stage_name, f);
        self
    }

    pub fn add_event<T>(&mut self) -> &mut Self
    where
        T: Send + Sync + 'static,
    {
        self.add_resource(Events::<T>::default())
            .add_system_to_stage(
                default_stage::EVENT_UPDATE,
                Events::<T>::build_update_system(),
            )
    }

    pub fn add_resource<T>(&mut self, resource: T) -> &mut Self
    where
        T: Send + Sync + 'static,
    {
        self.resources_mut().insert(resource);
        self
    }

    pub fn set_runner(&mut self, run_fn: impl Fn(App) + 'static) -> &mut Self {
        self.app_mut().runner = Some(Box::new(run_fn));
        self
    }

    pub fn load_plugin(&mut self, path: &str) -> &mut Self {
        let (_lib, plugin) = load_plugin(path);
        log::debug!("loaded plugin: {}", plugin.name());
        plugin.build(self);
        self
    }

    pub fn add_plugin<T>(&mut self, plugin: T) -> &mut Self
    where
        T: AppPlugin,
    {
        log::debug!("added plugin: {}", plugin.name());
        plugin.build(self);
        self
    }
}
