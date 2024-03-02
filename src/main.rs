//! Calibrate Magnetometer Visualizer
use bevy::{core::FrameCount, prelude::*, render::view::NoFrustumCulling, window::PresentMode};

mod camera;
mod render;

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
            render::PointMaterialPlugin,
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

fn setup(mut commands: Commands, meshes: ResMut<Assets<Mesh>>) {
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
    spawn_sphere(&mut commands, meshes, 1.0);
    camera::spawn_camera(commands);
}

fn spawn_sphere(commands: &mut Commands, mut meshes: ResMut<Assets<Mesh>>, r: f32) {
    let n = 50;

    commands.spawn((
        meshes.add(Cuboid::new(0.01, 0.01, 0.01)),
        SpatialBundle::INHERITED_IDENTITY,
        render::PointMaterialData(
            (0..=n/2)
                .flat_map(|ph| (0..n).map(move |th| (ph as f32 / (n/2) as f32, th as f32 / n as f32)))
                .map(|(ph, th)| {
                    let theta: f32 = 2.0 * std::f32::consts::PI * th;
                    let phi = (1.0 - 2.0 * ph).acos();
                    let x = r * phi.sin() * theta.cos();
                    let y = r * phi.sin() * theta.sin();
                    let z = r * phi.cos();
                    render::PointData {
                        position: Vec3::new(x, y, z),
                        scale: 1.0,
                        color: Color::BLUE.as_rgba_f32(),
                    }
                })
                .collect(),
        ),
        NoFrustumCulling,
    ));

}

fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.arrow(Vec3::ZERO, Vec3::X * 1.4, Color::RED);
    gizmos.arrow(Vec3::ZERO, Vec3::Y * 1.4, Color::GREEN);
    gizmos.arrow(Vec3::ZERO, Vec3::Z * 1.4, Color::BLUE);
}
