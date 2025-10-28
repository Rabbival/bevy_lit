use bevy::camera::visibility::RenderLayers;
use bevy::{
    camera::{ScalingMode, Viewport},
    color::palettes::tailwind::*,
    prelude::*,
    window::{PrimaryWindow, WindowResized, WindowResolution},
};
use bevy_lit::prelude::*;

const VIEWPORT_WORLD_SIZE: f32 = 200.0;
const WINDOW_INITIAL_SIZE_IN_PIXELS: u32 = 800;

#[derive(Component)]
struct CursorLight;

#[derive(Component)]
struct ViewportedCamera;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(
                            WINDOW_INITIAL_SIZE_IN_PIXELS,
                            WINDOW_INITIAL_SIZE_IN_PIXELS,
                        )
                        .with_scale_factor_override(1.0),
                        ..default()
                    }),
                    ..default()
                }),
            Lighting2dPlugin,
        ))
        .insert_resource(ClearColor(Color::from(GRAY_600)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (update_viewport_on_window_change, update_cursor_light),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = windows.single()
        && let Some(viewport) = calculate_viewport(window)
    {
        commands.spawn((
            Camera2d,
            RenderLayers::layer(1),
            ViewportedCamera,
            Projection::from(OrthographicProjection {
                scaling_mode: ScalingMode::AutoMin {
                    min_width: VIEWPORT_WORLD_SIZE * 2.0,
                    min_height: VIEWPORT_WORLD_SIZE * 2.0,
                },
                scale: 0.8,
                ..OrthographicProjection::default_2d()
            }),
            Camera {
                order: 0,
                viewport: Some(viewport),
                ..default()
            },
            Lighting2dSettings {
                raymarch: RaymarchSettings {
                    max_steps: 32,
                    jitter_contrib: 0.0,
                    sharpness: 64.,
                },
                scale: 0.125,
                ..default()
            },
            AmbientLight2d {
                intensity: 0.2,
                color: Color::from(BLUE_300),
            },
        ));
    }
    commands.spawn((
        Camera2d,
        RenderLayers::layer(0),
        Camera {
            order: 1,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedHorizontal {
                viewport_width: 320.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        // Lighting2dSettings {
        //     raymarch: RaymarchSettings {
        //         max_steps: 32,
        //         jitter_contrib: 0.0,
        //         sharpness: 64.,
        //     },
        //     scale: 0.125,
        //     ..default()
        // },
        // AmbientLight2d {
        //     intensity: 0.2,
        //     color: Color::from(BLUE_300),
        // },
    ));

    commands.spawn((
        CursorLight,
        TextureLight2d {
            image: asset_server.load("light_mask.png"),
            color: Color::from(YELLOW_400),
            intensity: 10.0,
            ..default()
        },
        Sprite::sized(Vec2::splat(10.0)),
        RenderLayers::layer(1),
    ));
}

fn calculate_viewport(window: &Window) -> Option<Viewport> {
    let window_width = window.resolution.width();
    let window_height = window.resolution.height();
    let width_scale_factor = window_width / WINDOW_INITIAL_SIZE_IN_PIXELS as f32;
    let height_scale_factor = window_height / WINDOW_INITIAL_SIZE_IN_PIXELS as f32;
    let position_x = window_width / 2.0 - VIEWPORT_WORLD_SIZE * width_scale_factor;
    let position_y = window_height / 2.0 - VIEWPORT_WORLD_SIZE * height_scale_factor;
    if position_x < 0.0 || position_y < 0.0 {
        None
    } else {
        let viewport = Viewport {
            physical_position: UVec2::new(position_x as u32, position_y as u32),
            physical_size: UVec2::new(
                (VIEWPORT_WORLD_SIZE * 2.0 * width_scale_factor) as u32,
                (VIEWPORT_WORLD_SIZE * 2.0 * height_scale_factor) as u32,
            ),
            ..default()
        };
        Some(viewport)
    }
}

fn update_viewport_on_window_change(
    mut window_resize_event_listener: MessageReader<WindowResized>,
    mut viewported_cameras: Query<&mut Camera, With<ViewportedCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if window_resize_event_listener.read().count() == 0 {
        return;
    }
    if let Ok(window) = windows.single()
        && let Ok(mut camera) = viewported_cameras.single_mut()
    {
        if window.width() < 1.0 || window.height() < 1.0 {
            camera.is_active = false;
            return;
        }
        if let Some(viewport) = calculate_viewport(window) {
            camera.is_active = true;
            camera.viewport = Some(viewport.clone());
        }
    }
}

fn update_cursor_light(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Lighting2dSettings>>,
    mut point_light_transform: Single<&mut Transform, With<CursorLight>>,
) {
    let (camera, camera_transform) = camera.into_inner();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate().extend(0.0))
    {
        point_light_transform.translation = world_position;
    }
}
