use bevy_asset::Handle;
use bevy_camera::visibility::Visibility;
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_math::Quat;
use bevy_reflect::prelude::*;
use bevy_transform::components::Transform;

/// A marker component for a light probe, which is a cuboid region that provides
/// global illumination to all fragments inside it.
///
/// Note that a light probe will have no effect unless the entity contains some
/// kind of illumination, which can either be an [`EnvironmentMapLight`] or an
/// `IrradianceVolume`.
///
/// The light probe range is conceptually a unit cube (1×1×1) centered on the
/// origin. The [`Transform`] applied to this entity can scale, rotate, or translate
/// that cube so that it contains all fragments that should take this light probe into account.
///
/// When multiple sources of indirect illumination can be applied to a fragment,
/// the highest-quality one is chosen. Diffuse and specular illumination are
/// considered separately, so, for example, Bevy may decide to sample the
/// diffuse illumination from an irradiance volume and the specular illumination
/// from a reflection probe. From highest priority to lowest priority, the
/// ranking is as follows:
///
/// | Rank | Diffuse              | Specular             |
/// | ---- | -------------------- | -------------------- |
/// | 1    | Lightmap             | Lightmap             |
/// | 2    | Irradiance volume    | Reflection probe     |
/// | 3    | Reflection probe     | View environment map |
/// | 4    | View environment map |                      |
///
/// Note that ambient light is always added to the diffuse component and does
/// not participate in the ranking. That is, ambient light is applied in
/// addition to, not instead of, the light sources above.
///
/// A terminology note: Unfortunately, there is little agreement across game and
/// graphics engines as to what to call the various techniques that Bevy groups
/// under the term *light probe*. In Bevy, a *light probe* is the generic term
/// that encompasses both *reflection probes* and *irradiance volumes*. In
/// object-oriented terms, *light probe* is the superclass, and *reflection
/// probe* and *irradiance volume* are subclasses. In other engines, you may see
/// the term *light probe* refer to an irradiance volume with a single voxel, or
/// perhaps some other technique, while in Bevy *light probe* refers not to a
/// specific technique but rather to a class of techniques. Developers familiar
/// with other engines should be aware of this terminology difference.
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component, Default, Debug, Clone)]
#[require(Transform, Visibility)]
pub struct LightProbe;

impl LightProbe {
    /// Creates a new light probe component.
    #[inline]
    pub fn new() -> Self {
        Self
    }
}

/// A pair of cubemap textures that represent the surroundings of a specific
/// area in space.
///
/// See `bevy_pbr::environment_map` for detailed information.
#[derive(Clone, Component, Reflect)]
#[reflect(Component, Default, Clone)]
pub struct EnvironmentMapLight {
    /// The blurry image that represents diffuse radiance surrounding a region.
    pub diffuse_map: Handle<Image>,

    /// The typically-sharper, mipmapped image that represents specular radiance
    /// surrounding a region.
    pub specular_map: Handle<Image>,

    /// Scale factor applied to the diffuse and specular light generated by this component.
    ///
    /// After applying this multiplier, the resulting values should
    /// be in units of [cd/m^2](https://en.wikipedia.org/wiki/Candela_per_square_metre).
    ///
    /// See also <https://google.github.io/filament/Filament.html#lighting/imagebasedlights/iblunit>.
    pub intensity: f32,

    /// World space rotation applied to the environment light cubemaps.
    /// This is useful for users who require a different axis, such as the Z-axis, to serve
    /// as the vertical axis.
    pub rotation: Quat,

    /// Whether the light from this environment map contributes diffuse lighting
    /// to meshes with lightmaps.
    ///
    /// Set this to false if your lightmap baking tool bakes the diffuse light
    /// from this environment light into the lightmaps in order to avoid
    /// counting the radiance from this environment map twice.
    ///
    /// By default, this is set to true.
    pub affects_lightmapped_mesh_diffuse: bool,
}

impl Default for EnvironmentMapLight {
    fn default() -> Self {
        EnvironmentMapLight {
            diffuse_map: Handle::default(),
            specular_map: Handle::default(),
            intensity: 0.0,
            rotation: Quat::IDENTITY,
            affects_lightmapped_mesh_diffuse: true,
        }
    }
}

/// A generated environment map that is filtered at runtime.
///
/// See `bevy_pbr::light_probe::generate` for detailed information.
#[derive(Clone, Component, Reflect)]
#[reflect(Component, Default, Clone)]
pub struct GeneratedEnvironmentMapLight {
    /// Source cubemap to be filtered on the GPU, size must be a power of two.
    pub environment_map: Handle<Image>,

    /// Scale factor applied to the diffuse and specular light generated by this
    /// component. Expressed in cd/m² (candela per square meter).
    pub intensity: f32,

    /// World-space rotation applied to the cubemap.
    pub rotation: Quat,

    /// Whether this light contributes diffuse lighting to meshes that already
    /// have baked lightmaps.
    pub affects_lightmapped_mesh_diffuse: bool,
}

impl Default for GeneratedEnvironmentMapLight {
    fn default() -> Self {
        GeneratedEnvironmentMapLight {
            environment_map: Handle::default(),
            intensity: 0.0,
            rotation: Quat::IDENTITY,
            affects_lightmapped_mesh_diffuse: true,
        }
    }
}

/// The component that defines an irradiance volume.
///
/// See `bevy_pbr::irradiance_volume` for detailed information.
///
/// This component requires the [`LightProbe`] component, and is typically used with
/// [`bevy_transform::components::Transform`] to place the volume appropriately.
#[derive(Clone, Reflect, Component, Debug)]
#[reflect(Component, Default, Debug, Clone)]
#[require(LightProbe)]
pub struct IrradianceVolume {
    /// The 3D texture that represents the ambient cubes, encoded in the format
    /// described in `bevy_pbr::irradiance_volume`.
    pub voxels: Handle<Image>,

    /// Scale factor applied to the diffuse and specular light generated by this component.
    ///
    /// After applying this multiplier, the resulting values should
    /// be in units of [cd/m^2](https://en.wikipedia.org/wiki/Candela_per_square_metre).
    ///
    /// See also <https://google.github.io/filament/Filament.html#lighting/imagebasedlights/iblunit>.
    pub intensity: f32,

    /// Whether the light from this irradiance volume has an effect on meshes
    /// with lightmaps.
    ///
    /// Set this to false if your lightmap baking tool bakes the light from this
    /// irradiance volume into the lightmaps in order to avoid counting the
    /// irradiance twice. Frequently, applications use irradiance volumes as a
    /// lower-quality alternative to lightmaps for capturing indirect
    /// illumination on dynamic objects, and such applications will want to set
    /// this value to false.
    ///
    /// By default, this is set to true.
    pub affects_lightmapped_meshes: bool,
}

impl Default for IrradianceVolume {
    #[inline]
    fn default() -> Self {
        IrradianceVolume {
            voxels: Handle::default(),
            intensity: 0.0,
            affects_lightmapped_meshes: true,
        }
    }
}
