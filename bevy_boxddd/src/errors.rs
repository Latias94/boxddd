use crate::messages::BoxdddErrorMessage;
use crate::resources::{BoxdddErrorPolicy, BoxdddPhysicsSettings};
use bevy_ecs::message::MessageWriter;

pub(crate) fn report_error(
    settings: &BoxdddPhysicsSettings,
    writer: &mut MessageWriter<'_, BoxdddErrorMessage>,
    message: BoxdddErrorMessage,
) {
    match settings.error_policy {
        BoxdddErrorPolicy::MessageOnly => {
            writer.write(message);
        }
        BoxdddErrorPolicy::MessageAndLog => {
            log::error!("{message:?}");
            writer.write(message);
        }
        BoxdddErrorPolicy::Panic => {
            panic!("{message:?}");
        }
    }
}
