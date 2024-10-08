/// https://bevy-cheatbook.github.io/platforms/wasm.html
///
/// must install wasm-pack
/// ```
/// rustup target add wasm32-unknown-unknown
/// cargo install trunk
/// cargo install wasm-bindgen-cli
/// ```
/// (webserver may not work)  run with : `cargo run --target wasm32-unknown-unknown`
/// run with `trunk serve --open`



/// export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-server-runner
/// cargo run --target wasm32-unknown-unknown
use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};


// use bevy::render::camera::Projection;
/// Helper resource for tracking our scene asset
#[derive(Resource)]
struct MyAssetPack {
    scene_handle: Handle<Scene>,
}


#[derive(Resource, Default)]
struct RotationState {
    rotating: bool,
}


fn mouse_input_system(
    buttons: Res<ButtonInput<MouseButton>>,
    mut rotation_state: ResMut<RotationState>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        rotation_state.rotating = true;
    }

    if buttons.just_released(MouseButton::Left) {
        rotation_state.rotating = false;
    }
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Web App".to_string(),
                // resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        // .add_plugins(DefaultPlugins)
        .init_resource::<RotationState>()
        .add_systems(Startup, (
            load_gltf,
            // add_people,
            setup_lighting_and_camera),
        )
        .add_systems(Update, (
            spawn_gltf_objects,
            // hello_world,
            // (update_people, greet_people).chain(),
            camera_rotation_system,
            // camera_zoom_system,
            update_camera_transform,
            mouse_input_system, // New system to handle mouse input
            camera_zoom_fov_system, // Updated zoom system
            camera_movement_system,

        ))
        .run();
}

//
// fn hello_world() {
//     println!("hello world!");
// }
//
//
// #[derive(Component)]
// struct Person;
//
//
// #[derive(Component)]
// struct Name(String);
//
//
// fn add_people(mut commands: Commands) {
//     commands.spawn((Person, Name("Elaina Proctor".to_string())));
//     commands.spawn((Person, Name("Renzo Hume".to_string())));
//     commands.spawn((Person, Name("Zayn a Nieves".to_string())));
// }
//
//
// fn greet_people(query: Query<&Name, With<Person>>) {
//     for name in &query {
//         println!("hello {}!", name.0);
//     }
// }
//
//
// fn update_people(mut query: Query<&mut Name, With<Person>>) {
//     for mut name in &mut query {
//         if name.0 == "Elaina Proctor" {
//             name.0 = "Elaina Hume".to_string();
//             break; // We don't need to change any other names.
//         }
//     }
// }


/// add light and camera
fn setup_lighting_and_camera(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0, // 調整光照強度
            ..default()
        },
        transform: Transform {
            rotation: Quat::from_euler(
                EulerRot::XYZ,
                -std::f32::consts::FRAC_PI_4,
                std::f32::consts::FRAC_PI_8,
                0.0,
            ),
            ..default()
        },
        ..default()
    });

    // default
    // commands.spawn((
    //     Camera3dBundle {
    //         transform: Transform::from_xyz(0.0, 0.0, 10.0)
    //             .looking_at(Vec3::ZERO, Vec3::Y),
    //         ..default()
    //     },
    //     CameraController::default(),
    // ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            projection: PerspectiveProjection {
                fov: 60.0_f32.to_radians(),
                ..default()
            }.into(),
            ..default()
        },
        CameraController::default(),
    ));
}


fn load_gltf(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // let scene_handle: Handle<Scene> = asset_server.load("/Users/terry/Downloads/bevyengine bevy \
    // v0.9.1 assets-models_FlightHelmet/FlightHelmet.gltf#Scene0");
    let scene_handle: Handle<Scene> = asset_server.load("3D_Isometric_BEDROOM.glb#Scene0");

    commands.insert_resource(MyAssetPack {
        scene_handle,
    });
}


fn spawn_gltf_objects(
    mut commands: Commands,
    my: Res<MyAssetPack>,
    scenes: Res<Assets<Scene>>,
    mut spawned: Local<bool>,
) {
    if *spawned {
        return; // if
    }


    if scenes.get(&my.scene_handle).is_some() {
        commands.spawn(SceneBundle {
            scene: my.scene_handle.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });
        *spawned = true;
    }
}


/// camera parameters
#[derive(Component)]
struct CameraController {
    // up_down rotation
    pitch: f32,
    // left_right rotation
    yaw: f32,
    // scale(discance)
    distance: f32,
    // mouse sensitivity
    mouse_sensitivity: f32,
    // scale sensitivity
    scroll_sensitivity: f32,
    min_distance: f32,
    max_distance: f32,
    // Field of View in degrees
    fov: f32,
    min_fov: f32,
    max_fov: f32,
    target: Vec3,
}


impl Default for CameraController {
    fn default() -> Self {
        CameraController {
            pitch: 0.0,
            yaw: 0.0,
            distance: 10.0,
            mouse_sensitivity: 0.1,
            scroll_sensitivity: 1.0,
            min_distance: 2.0,
            max_distance: 20.0,
            // Default FOV
            fov: 60.0,
            min_fov: 20.0,
            max_fov: 120.0,
            target: Vec3::ZERO,
        }
    }
}


fn camera_zoom_fov_system(
    mut scroll_events: EventReader<MouseWheel>,
    // mut query: Query<&mut CameraController, With<CameraController>>,
    mut query: Query<(&mut CameraController, &mut Transform), With<CameraController>>,

    mut cameras: Query<&mut Projection, With<Camera>>,
) {
    let mut scroll_delta = 0.0;
    for event in scroll_events.read() {
        scroll_delta += event.y;
    }

    if scroll_delta == 0.0 {
        return;
    }

    for (mut controller, mut transform)in query.iter_mut() {
        controller.fov -= scroll_delta * controller.scroll_sensitivity;
        controller.fov = controller.fov.clamp(controller.min_fov, controller.max_fov);

        // Update the camera's projection
        for mut projection in cameras.iter_mut() {
            if let Projection::Perspective(ref mut persp) = *projection {
                persp.fov = controller.fov.to_radians();
            }
        }

        // Update camera rotation
        let yaw_rotation = Quat::from_rotation_y(controller.yaw);
        let pitch_rotation = Quat::from_rotation_x(controller.pitch);
        let rotation = yaw_rotation * pitch_rotation;

        // transform.translation = rotation * Vec3::new(0.0, 0.0, controller.distance);
        // transform.look_at(Vec3::ZERO, Vec3::Y);

        transform.translation = controller.target+rotation * Vec3::new(0.0, 0.0, controller
            .distance);
        transform.look_at(controller.target, Vec3::Y);

    }
}


fn camera_rotation_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    rotation_state: Res<RotationState>,
    mut query: Query<(&mut CameraController, &mut Transform), With<CameraController>>,
) {
    if !rotation_state.rotating {
        return;
    }
    let mut delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    for (mut controller, mut transform) in query.iter_mut() {
        controller.yaw -= delta.x * controller.mouse_sensitivity * 0.1;
        controller.pitch -= delta.y * controller.mouse_sensitivity * 0.1;

        // Clamp the pitch to prevent flipping
        controller.pitch = controller.pitch.clamp(-89.9_f32.to_radians(), 89.9_f32.to_radians());

        // Update camera rotation
        let yaw_rotation = Quat::from_rotation_y(controller.yaw);
        let pitch_rotation = Quat::from_rotation_x(controller.pitch);
        let rotation = yaw_rotation * pitch_rotation;

        // transform.translation = rotation * Vec3::new(0.0, 0.0, controller.distance);
        // transform.look_at(Vec3::ZERO, Vec3::Y);

        transform.translation = controller.target+rotation * Vec3::new(0.0, 0.0, controller
            .distance);
        transform.look_at(controller.target, Vec3::Y);
    }
}

//
// fn camera_zoom_system(
//     mut scroll_events: EventReader<MouseWheel>,
//     mut query: Query<&mut CameraController, With<CameraController>>,
// ) {
//     let mut scroll_delta = 0.0;
//     for event in scroll_events.read() {
//         scroll_delta += event.y;
//     }
//
//     if scroll_delta == 0.0 {
//         return;
//     }
//
//     for mut controller in query.iter_mut() {
//         controller.distance -= scroll_delta * controller.scroll_sensitivity;
//         controller.distance = controller.distance.clamp(controller.min_distance, controller.max_distance);
//     }
// }


fn update_camera_transform(
    mut query: Query<(&CameraController, &mut Transform), With<CameraController>>,
) {
    for (controller, mut transform) in &mut query {
        let yaw_rotation = Quat::from_rotation_y(controller.yaw);
        let pitch_rotation = Quat::from_rotation_x(controller.pitch);
        let rotation = yaw_rotation * pitch_rotation;

        transform.translation = controller.target+rotation * Vec3::new(0.0, 0.0, controller
            .distance);
        transform.look_at(controller.target, Vec3::Y);
        // transform.look_at(Vec3::ZERO, Vec3::Y);
    }


}

//
// fn lock_cursor_system(
//     windows: Res<Windows>,
//     buttons: Res<ButtonInput<MouseButton>>,
// ) {
//     let window = windows.get_primary().unwrap();
//
//     if buttons.just_pressed(MouseButton::Left) {
//         window.set_cursor_grab_mode(bevy::window::CursorGrabMode::Locked);
//         window.set_cursor_visibility(false);
//     }
//
//     if buttons.just_released(MouseButton::Left) {
//         window.set_cursor_grab_mode(bevy::window::CursorGrabMode::None);
//         window.set_cursor_visibility(true);
//     }
// }


/// move camera
fn camera_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut CameraController, With<CameraController>>,
) {
    let mut direction = Vec3::ZERO;
    let speed = 5.0; // 调整移动速度

    if keyboard_input.pressed(KeyCode::KeyW ) || keyboard_input.pressed(KeyCode::ArrowUp) {
        direction += Vec3::Y;
    }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        direction -= Vec3::Y;
    }
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction -= Vec3::X;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::AltRight) {
        direction += Vec3::X;
    }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
    }

    for mut controller in query.iter_mut() {
        // trasform direction to world space
        let right = Quat::from_rotation_y(controller.yaw) * Vec3::X;
        let forward = Quat::from_rotation_y(controller.yaw) * Vec3::Z;
        let up = Vec3::Y;

        let movement = (right * direction.x + forward * direction.z + up * direction.y) * speed * time.delta_seconds();
        controller.target += movement;
    }
}

