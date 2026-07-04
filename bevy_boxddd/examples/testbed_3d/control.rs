use bevy::prelude::Resource;

pub const MIN_HERTZ: f64 = 10.0;
pub const MAX_HERTZ: f64 = 240.0;
pub const DEFAULT_HERTZ: f64 = 60.0;
pub const MIN_SUB_STEPS: i32 = 1;
pub const MAX_SUB_STEPS: i32 = 16;
pub const DEFAULT_SUB_STEPS: i32 = 4;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DebugDrawPreset {
    Off,
    Shapes,
    ShapesAndJoints,
    Contacts,
    Bounds,
}

impl DebugDrawPreset {
    pub const ALL: [Self; 5] = [
        Self::Off,
        Self::Shapes,
        Self::ShapesAndJoints,
        Self::Contacts,
        Self::Bounds,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Shapes => "Shapes",
            Self::ShapesAndJoints => "Shapes + joints",
            Self::Contacts => "Contacts",
            Self::Bounds => "Bounds",
        }
    }

    pub const fn is_enabled(self) -> bool {
        !matches!(self, Self::Off)
    }

    pub fn options(self) -> boxddd::DebugDrawOptions {
        let mut options = boxddd::DebugDrawOptions::default();
        options.draw_shapes = false;
        options.draw_joints = false;
        options.draw_joint_extras = false;
        options.draw_bounds = false;
        options.draw_mass = false;
        options.draw_contacts = false;
        options.draw_contact_features = false;
        options.draw_contact_normals = false;
        options.draw_contact_forces = false;
        options.draw_friction_forces = false;
        options.draw_islands = false;

        match self {
            Self::Off => {}
            Self::Shapes => {
                options.draw_shapes = true;
            }
            Self::ShapesAndJoints => {
                options.draw_shapes = true;
                options.draw_joints = true;
                options.draw_joint_extras = true;
            }
            Self::Contacts => {
                options.draw_shapes = true;
                options.draw_contacts = true;
                options.draw_contact_features = true;
                options.draw_contact_normals = true;
                options.draw_contact_forces = true;
                options.draw_friction_forces = true;
            }
            Self::Bounds => {
                options.draw_shapes = true;
                options.draw_bounds = true;
            }
        }

        options
    }

    pub fn toggled(self) -> Self {
        if self.is_enabled() {
            Self::Off
        } else {
            Self::ShapesAndJoints
        }
    }
}

impl Default for DebugDrawPreset {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Resource, Debug)]
pub struct TestbedState {
    pub scene_index: usize,
    pub paused: bool,
    pub debug_preset: DebugDrawPreset,
    pub gravity_enabled: bool,
    pub sleeping_enabled: bool,
    pub warm_starting_enabled: bool,
    pub continuous_enabled: bool,
    pub sub_step_count: i32,
    pub hertz: f64,
    pub single_step_pending: bool,
    pub single_step_active: bool,
}

impl Default for TestbedState {
    fn default() -> Self {
        Self {
            scene_index: 0,
            paused: false,
            debug_preset: DebugDrawPreset::default(),
            gravity_enabled: true,
            sleeping_enabled: true,
            warm_starting_enabled: true,
            continuous_enabled: true,
            sub_step_count: DEFAULT_SUB_STEPS,
            hertz: DEFAULT_HERTZ,
            single_step_pending: false,
            single_step_active: false,
        }
    }
}

impl TestbedState {
    pub fn clamp_controls(&mut self) {
        self.sub_step_count = self.sub_step_count.clamp(MIN_SUB_STEPS, MAX_SUB_STEPS);
        self.hertz = self.hertz.clamp(MIN_HERTZ, MAX_HERTZ);
    }

    pub fn fixed_timestep_seconds(&self) -> f64 {
        1.0 / self.hertz.clamp(MIN_HERTZ, MAX_HERTZ)
    }

    pub fn request_single_step(&mut self) {
        if self.paused {
            self.single_step_pending = true;
        }
    }

    pub fn cancel_single_step(&mut self) {
        self.single_step_pending = false;
        self.single_step_active = false;
    }
}
