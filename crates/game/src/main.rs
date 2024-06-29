#![allow(clippy::needless_pass_by_value)]
use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
    window::CursorGrabMode,
};
use hexasphere_organized::Coordinate;

fn setup_sphere_object(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const SUBDIV: usize = 32;
    let geometry = hexasphere::shapes::IcoSphere::new(SUBDIV, |_| ());

    let indices = geometry.get_all_indices();

    let geometry_points = geometry.raw_points();

    let (organized, new_geometry, color_data) = hexasphere_organized::Hexasphere::make_and_dual(
        SUBDIV,
        &indices,
        geometry_points,
        |geometry_data| {
            (0..geometry_data.points.len())
                .map(|_| [1.0; 4])
                .collect::<Vec<_>>()
        },
        |index, edges, coord, geometry_data, color_data| {
            let color = match coord {
                Coordinate::Top => Vec4::new(1.0, 1.0, 0.0, 1.0),
                Coordinate::Bottom => Vec4::new(1.0, 0.0, 1.0, 1.0),
                Coordinate::Inside { chunk, short, long } => {
                    let chunk = f32::from(chunk);
                    let short = short as f32;
                    let long = long as f32;
                    Vec4::new(chunk, short, long, 1.0).normalize()
                }
            };
            // println!("index: {index}, coord: {coord:?} color: {color:?}, edges: {edges:?}");
            color_data[index as usize] = color.into();
            (index, edges)
        },
    );

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_indices(Indices::U32(new_geometry.indices));
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(
            new_geometry
                .points
                .iter()
                .copied()
                .map(std::convert::Into::into)
                .collect(),
        ),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(
            new_geometry
                .normals
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
        ),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2((0..color_data.len()).map(|_| [0.0; 2]).collect()),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::Float32x4(color_data),
    );

    let scaling_factor = 1000.0;
    let mut transform = Transform::from_xyz(0.0, 0.0, 0.0);
    transform.scale = Vec3::new(scaling_factor, scaling_factor, scaling_factor);

    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: false,
            ..Default::default()
        }),
        transform,
        ..default()
    });

    // Now use `organized` to manage coordinates, etc.
    // Now use `organized` to manage coordinates, etc.
    let hex = organized.get_many([Coordinate::Top]);
    let first = hex.first().unwrap();
    println!("{:?}", first);

    // save access to it for later
    commands.insert_resource(organized);
}

fn setup(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
) {
    // Start the player away from the sphere
    let starting_position = Vec3::new(0.0, 0.0, 1010.0);
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(
            starting_position.x,
            starting_position.y,
            starting_position.z,
        )
        .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn camera_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &Camera)>,
) {
    // Adjust the movement to be relative to the camera's rotation
    for (mut transform, _) in &mut query {
        let mut direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction -= transform.rotation * Vec3::Z;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction += transform.rotation * Vec3::Z;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction -= transform.rotation * Vec3::X;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += transform.rotation * Vec3::X;
        }
        if keyboard_input.pressed(KeyCode::KeyQ) {
            direction += transform.rotation * Vec3::Y;
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            direction -= transform.rotation * Vec3::Y;
        }
        transform.translation += direction * 5.;
    }
}

fn camera_look(
    mut mouse_events: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &Camera)>,
) {
    for event in mouse_events.read() {
        for (mut transform, _) in &mut query {
            transform.rotation *= Quat::from_rotation_y(-event.delta.x * 0.001);
            transform.rotation *= Quat::from_rotation_x(-event.delta.y * 0.001);
        }
    }
}

fn grab_mouse(mut windows: Query<&mut Window>, mouse: Res<ButtonInput<MouseButton>>) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }
    if mouse.just_released(MouseButton::Left) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_sphere_object)
        .add_systems(Update, (camera_movement, camera_look))
        .add_systems(Update, grab_mouse);
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, bevy::window::close_on_esc);
    app.run();
}
