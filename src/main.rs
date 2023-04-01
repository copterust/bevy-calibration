//! Calibrate Magnetometer Visualizer
//!
//! Particles with help from https://github.com/rust-adventure/bevy-examples/tree/main/examples/pointcloud
//! Camera from https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::window::{PrimaryWindow, Window};
use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_serial::{SerialPlugin, SerialReadEvent};
use rand::distributions::{Distribution, Uniform};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(SerialPlugin::new("COM3", 460800))
        .add_plugin(MaterialPlugin::<ParticlesMaterial>::default())
        .add_plugin(MaterialPlugin::<LineMaterial>::default())
        .insert_resource(ClearColor(Color::hex("0f0f0f").unwrap()))
        .add_system(update_time_for_particles_material)
        .add_system(read_serial)
        .add_system(pan_orbit_camera)
        .add_system(draw_ui)
        .add_startup_system(setup)
        .run();
}

fn draw_ui(mut contexts: EguiContexts) {
    egui::Window::new("Calibration").show(
        contexts.ctx_mut(),
        |ui| {
            if ui.button("Done").clicked() {}
        },
    );
}

fn read_serial(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ParticlesMaterial>>,
    mut ev_serial: EventReader<SerialReadEvent>,
) {
    let mut mesh = Mesh::new(PrimitiveTopology::PointList);
    let mut positions = vec![];
    let mut colors = vec![];

    for SerialReadEvent(_label, buffer) in ev_serial.iter() {
        let s = String::from_utf8(buffer.clone()).unwrap();
        if let Ok(mag) = parse_serial(s) {
            positions.push([mag[0], mag[1], mag[2]]);
            colors.push([1.0, 1.0, 1.0, 1.0]);
        }
    }

    if positions.len() > 0 {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        commands.spawn(MaterialMeshBundle {
            mesh: meshes.add(mesh),
            material: materials.add(ParticlesMaterial { time: 0.0 }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });
    }
}

fn parse_serial(input: String) -> Result<[f32; 3], ()> {
    if let Some(index) = input.rfind('[') {
        let last_list = &input[index + 1..input.len() - 5];
        let numbers = last_list
            .split(", ")
            .map(|s| s.parse::<f32>().unwrap())
            .collect::<Vec<_>>();
        if numbers.len() >= 3 {
            let (mx, my, mz) = (
                numbers[numbers.len() - 3],
                numbers[numbers.len() - 2],
                numbers[numbers.len() - 1],
            );
            return Ok([mx, my, mz]);
        } else {
            println!("Error: not enough numbers found");
        }
    } else {
        println!("Error: no list found");
    }
    Err(())
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ParticlesMaterial>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
) {
    let mut mesh = Mesh::new(PrimitiveTopology::PointList);
    let uniform01 = Uniform::from(0.0..1.0);
    let mut rng = rand::thread_rng();
    let mut positions = vec![];
    let mut colors = vec![];
    for _ in 0..1000 {
        let theta: f32 = 2.0 * std::f32::consts::PI * uniform01.sample(&mut rng);
        let phi = (1.0 - 2.0 * uniform01.sample(&mut rng)).acos();
        let x = 600. * phi.sin() * theta.cos();
        let y = 600. * phi.sin() * theta.sin();
        let z = 600. * phi.cos();
        positions.push([x, y, z]);
        colors.push([0.0, 0.0, 0.5, 1.0]);
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

    spawn_camera(&mut commands);
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(mesh),
        material: materials.add(ParticlesMaterial { time: 0.0 }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
    // Axis
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![
                (Vec3::ZERO, Vec3::new(1000.0, 0.0, 0.0)),
            ],
        })),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        material: line_materials.add(LineMaterial {
            color: Color::RED,
        }),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![
                (Vec3::ZERO, Vec3::new(0.0, 1000.0, 0.0)),
            ],
        })),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        material: line_materials.add(LineMaterial {
            color: Color::GREEN,
        }),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![
                (Vec3::ZERO, Vec3::new(0.0, 0.0, 1000.0)),
            ],
        })),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        material: line_materials.add(LineMaterial {
            color: Color::BLUE,
        }),
        ..default()
    });
}

#[derive(Default, AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "050ce6ac-080a-4d8c-b6b5-b5bab7560d8f"]
struct LineMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

/// A list of lines with a start and end position
#[derive(Debug, Clone)]
pub struct LineList {
    pub lines: Vec<(Vec3, Vec3)>,
}

impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        // This tells wgpu that the positions are list of lines
        // where every pair is a start and end point
        let mut mesh = Mesh::new(PrimitiveTopology::LineList);

        let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh
    }
}

/// A list of points that will have a line drawn between each consecutive points
#[derive(Debug, Clone)]
pub struct LineStrip {
    pub points: Vec<Vec3>,
}

impl From<LineStrip> for Mesh {
    fn from(line: LineStrip) -> Self {
        // This tells wgpu that the positions are a list of points
        // where a line will be drawn between each consecutive point
        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, line.points);
        mesh
    }
}

impl Material for ParticlesMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/particles.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/particles.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f85ae190-dd13-4f3c-9775-865c84c021fe"]
pub struct ParticlesMaterial {
    #[uniform(0)]
    time: f32,
}

fn update_time_for_particles_material(
    mut materials: ResMut<Assets<ParticlesMaterial>>,
    time: Res<Time>,
) {
    for material in materials.iter_mut() {
        material.1.time = time.raw_elapsed_seconds() as f32;
    }
}

/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
struct PanOrbitCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
fn pan_orbit_camera(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Middle;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll: f32 = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.iter() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.iter() {
            pan += ev.delta;
        }
    }
    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let Ok(window) = windows.get_single() else {
                return;
            };
            let window = get_primary_window_size(window);
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            let Ok(window) = windows.get_single() else {
                return;
            };
            let window = get_primary_window_size(window);
            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            }
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }

    // consume any remaining events, so they don't pile up if we don't need them
    // (and also to avoid Bevy warning us about not checking events every frame update)
    ev_motion.clear();
}

fn get_primary_window_size(primary: &Window) -> Vec2 {
    let window = Vec2::new(primary.width() as f32, primary.height() as f32);
    window
}

/// Spawn a camera like this
fn spawn_camera(commands: &mut Commands) {
    let translation = Vec3::new(2500., 2500., 2000.);
    let radius = translation.length();

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        PanOrbitCamera {
            radius,
            ..Default::default()
        },
    ));
}
