//! Calibrate Magnetometer Visualizer

use bevy::render::render_resource::ShaderRef;
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{mesh::PrimitiveTopology, render_resource::AsBindGroup},
};
use bevy_serial::{SerialPlugin, SerialReadEvent};
use rand::distributions::{Distribution, Uniform};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SerialPlugin::new("COM3", 460800))
        .insert_resource(ClearColor(Color::hex("0f0f0f").unwrap()))
        .add_plugin(MaterialPlugin::<ParticlesMaterial>::default())
        .add_startup_system(setup)
        .add_system(update_time_for_particles_material)
        .add_system(read_serial)
        .run();
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
        println!("{}", s);
        if let Ok(mag) = parse_serial(s) {
            println!("{:?}", mag);
            positions.push([mag[0], mag[1], mag[2]]);
            colors.push([0.0, 0.0, 1.0, 1.0]);
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
            .map(|s| {
                println!("{}", s);
                s.parse::<f32>().unwrap()
            })
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
        colors.push([0.2, 0.2, 0.2, 1.0]);
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(2500., 2500., 2000.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(mesh),
        material: materials.add(ParticlesMaterial { time: 0.0 }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
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
