use crate::messages::{
    BoxdddBodyMoveMessage, BoxdddContactBeginMessage, BoxdddContactEndMessage,
    BoxdddContactHitMessage, BoxdddErrorMessage, BoxdddOperation, BoxdddSensorBeginMessage,
    BoxdddSensorEndMessage,
};
use crate::resources::{BoxdddErrorPolicy, BoxdddPhysicsContext, BoxdddPhysicsSettings};
use crate::systems::{
    apply_body_controls, cleanup_removed_bodies, cleanup_removed_colliders, cleanup_removed_joints,
    create_missing_bodies, create_missing_joints, create_missing_shapes, publish_physics_messages,
    step_world, sync_bevy_transforms_to_boxddd, sync_boxddd_transforms_to_bevy,
};
use bevy_app::{App, FixedUpdate, Plugin};
use bevy_ecs::schedule::{ApplyDeferred, IntoScheduleConfigs};
use bevy_time::{Fixed, Time};

#[derive(Clone, Debug, Default)]
pub struct BoxdddPhysicsPlugin {
    settings: BoxdddPhysicsSettings,
}

impl BoxdddPhysicsPlugin {
    pub fn new(settings: BoxdddPhysicsSettings) -> Self {
        Self { settings }
    }
}

impl Plugin for BoxdddPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BoxdddErrorMessage>()
            .add_message::<BoxdddBodyMoveMessage>()
            .add_message::<BoxdddContactBeginMessage>()
            .add_message::<BoxdddContactEndMessage>()
            .add_message::<BoxdddContactHitMessage>()
            .add_message::<BoxdddSensorBeginMessage>()
            .add_message::<BoxdddSensorEndMessage>();

        app.insert_resource(self.settings.clone());

        if let Some(seconds) = self.settings.fixed_timestep_seconds {
            if seconds.is_finite() && seconds > 0.0 {
                app.insert_resource(Time::<Fixed>::from_seconds(seconds));
            } else {
                let message = BoxdddErrorMessage {
                    operation: BoxdddOperation::ConfigureFixedTimestep,
                    entity: None,
                    error: boxddd::Error::InvalidArgument,
                };
                report_startup_error(app, self.settings.error_policy, message);
                app.insert_resource(Time::<Fixed>::default());
            }
        }

        let context = match BoxdddPhysicsContext::new(&self.settings) {
            Ok(context) => context,
            Err(error) => {
                let message = BoxdddErrorMessage {
                    operation: BoxdddOperation::CreateWorld,
                    entity: None,
                    error,
                };
                report_startup_error(app, self.settings.error_policy, message);
                BoxdddPhysicsContext::disabled()
            }
        };

        app.insert_non_send(context);

        app.add_systems(
            FixedUpdate,
            (
                cleanup_removed_joints,
                cleanup_removed_colliders,
                cleanup_removed_bodies,
                create_missing_bodies,
                ApplyDeferred,
                create_missing_shapes,
                create_missing_joints,
                apply_body_controls,
                sync_bevy_transforms_to_boxddd,
                step_world,
                publish_physics_messages,
                sync_boxddd_transforms_to_bevy,
            )
                .chain(),
        );
    }
}

fn report_startup_error(app: &mut App, policy: BoxdddErrorPolicy, message: BoxdddErrorMessage) {
    match policy {
        BoxdddErrorPolicy::MessageOnly => {
            app.world_mut().write_message(message);
        }
        BoxdddErrorPolicy::MessageAndLog => {
            log::error!("{message:?}");
            app.world_mut().write_message(message);
        }
        BoxdddErrorPolicy::Panic => {
            panic!("{message:?}");
        }
    }
}
