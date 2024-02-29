//! Calibrate Magnetometer Visualizer
use bevy::{core::FrameCount, prelude::*, window::PresentMode};

mod camera;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Magnetometer Calibration".into(),
                    resolution: (640., 480.).into(),
                    present_mode: PresentMode::AutoVsync,
                    prevent_default_event_handling: false,
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: false,
                        ..Default::default()
                    },
                    visible: false,
                    ..default()
                }),
                ..default()
            }),
            // LogDiagnosticsPlugin::default(),
            // FrameTimeDiagnosticsPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (make_visible, draw_gizmos, camera::pan_orbit_camera),
        )
        .run();
}

fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    if frames.0 == 3 {
        // At this point the gpu is ready to show the app so we can make the window visible.
        window.single_mut().visible = true;
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(
        TextBundle::from_section(
            "Controls\n\
                    Pan: LMB; Orbit: RMB, Zoom: Scroll",
            TextStyle {
                font_size: 20.,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );
    camera::spawn_camera(commands);
}

fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.arrow(Vec3::ZERO, Vec3::X * 1.4, Color::RED);
    gizmos.arrow(Vec3::ZERO, Vec3::Y * 1.4, Color::GREEN);
    gizmos.arrow(Vec3::ZERO, Vec3::Z * 1.4, Color::BLUE);
    gizmos.sphere(Vec3::ZERO, Quat::IDENTITY, 0.5, Color::DARK_GRAY);
}
