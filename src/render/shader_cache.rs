use std::hash::BuildHasher;

use bevy::{
    asset::{Assets, Handle},
    ecs::{change_detection::ResMut, resource::Resource},
    log::{debug, trace},
    platform::collections::HashMap,
    shader::Shader,
};

/// Cache of baked shaders variants.
///
/// Baked shader variants are shaders where the placeholders `{{PLACEHOLDER}}`
/// present in the WESL template code have been replaced by actual WGSL code,
/// making them a valid shader from the point of view of the Bevy renderer.
///
/// Shaders present in the cache are allocated [`Shader`] resources. Note that a
/// [`Shader`] resource is further preprocessed by the WESL compiler to resolve
/// imports and conditional compilation attributes (`@if`); to this extent,
/// entries are not compilable WGSL as is.
#[derive(Default, Resource)]
pub struct ShaderCache {
    /// Map of allocated shader resources from their baked shader code.
    cache: HashMap<String, Handle<Shader>>,
}

impl ShaderCache {
    /// Get an existing baked shader variant, or insert it into the cache and
    /// allocate a new [`Shader`] resource for it.
    ///
    /// Returns the [`Shader`] resource associated with `source`.
    pub fn get_or_insert(
        &mut self,
        filename: &str,
        suffix: &str,
        source: &str,
        shaders: &mut ResMut<Assets<Shader>>,
    ) -> Handle<Shader> {
        if let Some(handle) = self.cache.get(source) {
            handle.clone()
        } else {
            let hash = bevy::platform::hash::FixedHasher.hash_one(source);
            // The path determines the WESL module path of the shader, so restrict it to
            // characters valid in an identifier. It's otherwise only used for debugging.
            let filename: String = filename
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect();
            let shader = Shader::from_wesl(
                source.to_string(),
                format!("hanabi/{}_{}_{}.wesl", filename, suffix, hash),
            );
            trace!(
                "Shader path={} import_path={:?} imports={:?}",
                shader.path,
                shader.import_path,
                shader.imports
            );
            let shader_path = shader.path.clone();
            let handle = shaders.add(shader);
            debug!(
                "Inserted new configured shader {}: {:?}\n{}",
                shader_path, handle, source
            );
            self.cache.insert(source.to_string(), handle.clone());
            handle
        }
    }
}
