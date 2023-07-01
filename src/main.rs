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
        mesh::{MeshVertexBufferLayout, PrimitiveTopology, VertexAttributeValues},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_serial::{SerialPlugin, SerialReadEvent};
use rand::distributions::{Distribution, Uniform};
use serde::Deserialize;
use serde_json;
use nalgebra::{Matrix3, Vector3};

mod math;

// Get yours at https://www.ngdc.noaa.gov/geomag/calculators/magcalc.shtml#igrfwmm
const F: f32 = 486.027;

#[derive(Resource)]
struct Calibration {
    a_1: Matrix3<f64>,
    b: Vector3<f64>,
}

#[derive(Default, Deserialize)]
pub struct Sample {
    pub dt: f32,
    pub accel: [f32; 3],
    pub gyro: [f32; 3],
    pub cal_mag: [f32; 3],
    pub state: [[f32; 7]; 1],
    pub raw_mag: [f32; 3],
}

impl Default for Calibration {
    fn default() -> Self {
        Calibration {
            a_1: Matrix3::identity(),
            b: Vector3::zeros(),
        }
    }
}

#[derive(Resource, PartialEq)]
enum AppState {
    Collect,
    Calibrate,
}

#[derive(Resource, PartialEq)]
enum SampleKind {
    Raw,
    Cal
}


impl Default for AppState {
    fn default() -> Self {
        AppState::Collect
    }
}

fn main() {
    let name = std::env::args().skip(1).next();
    let name = name.as_deref().unwrap_or("tilt1.txt");

    let kind = std::env::args().skip(2).next().unwrap_or("raw".to_string());
    let kind = if kind == "cal" {
        SampleKind::Cal
    } else {
        SampleKind::Raw
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(SerialPlugin::new(name, 460800))
        .add_plugin(MaterialPlugin::<ParticlesMaterial>::default())
        .add_plugin(MaterialPlugin::<LineMaterial>::default())
        .insert_resource(ClearColor(Color::hex("0f0f0f").unwrap()))
        .insert_resource(AppState::Collect)
        .insert_resource(Calibration::default())
        .add_system(update_time_for_particles_material)
        .insert_resource(kind)
        .add_system(read_serial)
        .add_system(pan_orbit_camera)
        .add_system(draw_ui)
        .add_startup_system(setup)
        .run();
}

fn draw_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<AppState>,
    mut query: Query<&Handle<Mesh>, With<RawMeasurements>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut calibration: ResMut<Calibration>,
) {
    egui::Window::new("Calibration").show(contexts.ctx_mut(), |ui| {
        if AppState::Collect == *state {
            if ui.button("Done").clicked() {
                *state = AppState::Calibrate;
                let handle = query.get_single_mut().expect("Raw Measurements mesh to be");
                let mesh = meshes.get_mut(handle).expect("getting mesh");
                let attribute_positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION);
                if let Some(VertexAttributeValues::Float32x3(positions)) = attribute_positions {
                    let samples: Vec<[f64; 3]> = positions
                        .into_iter()
                        .map(|arr| [arr[0] as f64, arr[1] as f64, arr[2] as f64])
                        .collect();
                    let (m, n, d) = math::ellipsoid_fit(&(samples as Vec<[f64; 3]>));
                    let (a_1, b) = math::ellipsoid_to_calibration(m, n, d, F as f64);
                    calibration.a_1 = a_1;
                    calibration.b = b;
                }
            }
        }
    });
}

#[derive(Component)]
struct RawMeasurements;

fn read_serial(
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<&Handle<Mesh>, With<RawMeasurements>>,
    mut ev_serial: EventReader<SerialReadEvent>,
    state: Res<AppState>,
    calibration: Res<Calibration>,
    kind: Res<SampleKind>
) {
    let handle = query.get_single_mut().expect("Raw Measurements mesh to be");
    let mesh = meshes.get_mut(handle).expect("getting mesh");

    let attribute_positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION);
    let attribute_colors = mesh.attribute(Mesh::ATTRIBUTE_COLOR);

    let mut positions =
        if let Some(VertexAttributeValues::Float32x3(previous_positions)) = attribute_positions {
            previous_positions.clone()
        } else {
            vec![]
        };

    let mut colors =
        if let Some(VertexAttributeValues::Float32x4(previous_colors)) = attribute_colors {
            previous_colors.clone()
        } else {
            vec![]
        };

    let mut bubu = Box::new(Sample::default());

    for SerialReadEvent(_label, buffer) in ev_serial.iter() {
        let s = match String::from_utf8(buffer.clone()) {
            Ok(x) => x,
            Err(_) => continue,
        };

        *bubu = match serde_json::from_str(&s) {
            Ok(k) => k,
            Err(_) => continue,
        };
        let cal = match *kind {
            SampleKind::Raw => bubu.raw_mag,
            SampleKind::Cal => bubu.cal_mag,
        };

        // if let Ok(mag) = parse_serial(s) {
            // let cal = math::calibrated_sample(
            //     &mag,
            //     &calibration.a_1.map(|x| x as f32),
            //     &calibration.b.map(|x| x as f32),
            // );
        positions.push([cal[0], cal[1], cal[2]]);

        if AppState::Collect == *state {
            colors.push([1.0, 0.0, 0.0, 1.0]);
        } else {
            colors.push([0.0, 1.0, 0.0, 1.0])
        }
        // }
    }

    if positions.len() > 0 {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
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
        let x = F * phi.sin() * theta.cos();
        let y = F * phi.sin() * theta.sin();
        let z = F * phi.cos();
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
    // Uncalibrated Point cloud
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::new(PrimitiveTopology::PointList)),
            material: materials.add(ParticlesMaterial { time: 0.0 }),
            ..default()
        },
        RawMeasurements,
    ));
    // Axis
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![(Vec3::ZERO, Vec3::new(F / 2., 0.0, 0.0))],
        })),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        material: line_materials.add(LineMaterial { color: Color::RED }),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![(Vec3::ZERO, Vec3::new(0.0, F / 2., 0.0))],
        })),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        material: line_materials.add(LineMaterial {
            color: Color::GREEN,
        }),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![(Vec3::ZERO, Vec3::new(0.0, 0.0, F / 2.))],
        })),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        material: line_materials.add(LineMaterial { color: Color::BLUE }),
        ..default()
    });
    // North
    let i = 66.8579f32.to_radians();
    let d = -5.9791f32.to_radians();
    let x: f32 = 1.2 * F * i.cos() * d.cos();
    let y: f32 = 1.2 * F * i.cos() * d.sin();
    let z: f32 = 1.2 * F * i.sin();
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![(Vec3::ZERO, Vec3::new(x, y, z))],
        })),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        material: line_materials.add(LineMaterial {
            color: Color::YELLOW,
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
