use bevy::{app::PluginGroup, prelude::*};

#[cfg(target_os = "windows")]
use bevy::render::{
    RenderPlugin,
    settings::{Backends, WgpuSettings},
};

pub fn teaching_default_plugins(title: &'static str) -> impl PluginGroup {
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: title.into(),
            ..default()
        }),
        ..default()
    };

    #[cfg(target_os = "windows")]
    {
        DefaultPlugins.set(window_plugin).set(RenderPlugin {
            render_creation: WgpuSettings {
                backends: Some(Backends::from_env().unwrap_or(Backends::DX12)),
                ..default()
            }
            .into(),
            ..default()
        })
    }

    #[cfg(not(target_os = "windows"))]
    {
        DefaultPlugins.set(window_plugin)
    }
}
