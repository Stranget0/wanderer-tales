use crate::prelude::*;

pub trait DebugFlagsExt {
    fn group(&self) -> &'static str;
    fn as_str(&self) -> &'static str;
}

#[derive(Resource, Default)]
pub struct DebugFlags {
    pub(crate) flags: hashbrown::HashMap<&'static str, bool>,
    pub(crate) groups: hashbrown::HashMap<&'static str, Vec<&'static str>>,
    pub(crate) has_changed: bool,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<DebugFlags>()
        .add_systems(Update, mark_has_changed_off.in_set(GameSet::PostUpdate));
}

pub(crate) fn mark_has_changed_off(mut flags: ResMut<DebugFlags>) {
    flags.has_changed = false;
}

impl DebugFlags {
    pub fn get(&self, flag: &impl DebugFlagsExt) -> bool {
        self.flags.get(flag.as_str()).copied().unwrap_or(false)
    }

    pub fn register(&mut self, flag: &impl DebugFlagsExt) -> &mut Self {
        let group = flag.group();
        let flag = flag.as_str();

        self.flags.insert(flag, false);
        if let Some(v) = self.groups.get_mut(group) {
            v.push(flag);
        } else {
            self.groups.insert(group, vec![flag]);
        }
        self
    }
}

pub fn register_debug_flags(app: &mut App, flags: Vec<impl DebugFlagsExt>) {
    let world = app.world_mut();
    world.init_resource::<DebugFlags>();

    let mut flags_map = world.get_resource_mut::<DebugFlags>().unwrap();
    for flag in flags.iter() {
        flags_map.register(flag);
    }
}

pub fn debug_flag_enabled(flag: &impl DebugFlagsExt) -> impl Fn(Res<DebugFlags>) -> bool + '_ {
    move |flags: Res<DebugFlags>| flags.get(flag)
}

pub fn debug_flags_changed(flags: Res<DebugFlags>) -> bool {
    flags.has_changed || flags.is_added()
}
