use std::env;

use bevy::prelude::*;

use jpeg2k::loader::*;

fn main() {
  dotenv::dotenv().ok();
  env_logger::init();

  App::build()
    .add_plugins(DefaultPlugins)
    .add_plugin(Jpeg2KPlugin)
    .add_startup_system(setup.system())
    .run();
}

fn setup(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  let name = env::args().nth(1)
    .unwrap_or_else(|| "example.j2k".to_string());

  let texture_handle = asset_server.load(name.as_str());
  // ui camera
  commands.spawn_bundle(UiCameraBundle::default());
  // root node
  commands
    .spawn_bundle(NodeBundle {
      style: Style {
        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
        justify_content: JustifyContent::SpaceBetween,
        ..Default::default()
      },
      material: materials.add(Color::NONE.into()),
      ..Default::default()
    })
    .with_children(|parent| {
      // bevy logo (image)
      parent.spawn_bundle(ImageBundle {
        style: Style {
          size: Size::new(Val::Auto, Val::Percent(100.0)),
          ..Default::default()
        },
        material: materials
          .add(texture_handle.into()),
        ..Default::default()
      });
    });
}
