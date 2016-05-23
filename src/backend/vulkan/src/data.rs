// Copyright 2016 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use gfx_core::factory::{Bind, MapAccess, Usage, LayerError};
use gfx_core::format::{SurfaceType, ChannelType, Swizzle, ChannelSource};
use gfx_core::tex::{FilterMethod, Kind, Layer, PackedColor, WrapMode};
use gfx_core::state;
use vk;


pub fn map_image_type(kind: Kind) -> vk::ImageType {
   match kind {
        Kind::D1(..) | Kind::D1Array(..) => vk::IMAGE_TYPE_1D,
        Kind::D2(..) | Kind::D2Array(..) => vk::IMAGE_TYPE_2D,
        Kind::D3(..) => vk::IMAGE_TYPE_3D,
        Kind::Cube(..) | Kind::CubeArray(..) => vk::IMAGE_TYPE_2D,
    }
}

pub fn map_image_view_type(kind: Kind, layer: Option<Layer>) -> Result<vk::ImageViewType, LayerError> {
    match (kind, layer) {
        (Kind::D1(..), Some(_)) | (Kind::D2(..), Some(_)) | (Kind::D3(..), Some(_)) |
        (Kind::Cube(..), Some(_)) => Err(LayerError::NotExpected(kind)),
        (Kind::D1Array(_, n),       Some(l)) if n<=l => Err(LayerError::OutOfBounds(l, n)),
        (Kind::D2Array(_, _, n, _), Some(l)) if n<=l => Err(LayerError::OutOfBounds(l, n)),
        (Kind::CubeArray(_, n),     Some(l)) if n<=l => Err(LayerError::OutOfBounds(l, n)),
        (Kind::D1(..), None) | (Kind::D1Array(..), Some(_)) => Ok(vk::IMAGE_VIEW_TYPE_1D),
        (Kind::D1Array(..), None) => Ok(vk::IMAGE_VIEW_TYPE_1D_ARRAY),
        (Kind::D2(..), None) | (Kind::D2Array(..), Some(_)) => Ok(vk::IMAGE_VIEW_TYPE_2D),
        (Kind::D2Array(..), None) => Ok(vk::IMAGE_VIEW_TYPE_2D_ARRAY),
        (Kind::D3(..), None) => Ok(vk::IMAGE_VIEW_TYPE_3D),
        (Kind::Cube(..), None) | (Kind::CubeArray(..), Some(_)) => Ok(vk::IMAGE_VIEW_TYPE_CUBE),
        (Kind::CubeArray(..), None) => Ok(vk::IMAGE_VIEW_TYPE_CUBE_ARRAY),
    }
}

pub fn map_image_aspect(surface: SurfaceType, channel: ChannelType, is_target: bool) -> vk::ImageAspectFlags {
    match surface {
        SurfaceType::D16 | SurfaceType::D24 | SurfaceType::D24_S8 | SurfaceType::D32 => match (is_target, channel) {
            (true, _) => vk::IMAGE_ASPECT_DEPTH_BIT | vk::IMAGE_ASPECT_STENCIL_BIT,
            (false, ChannelType::Float) => vk::IMAGE_ASPECT_DEPTH_BIT,
            (false, ChannelType::Uint)  => vk::IMAGE_ASPECT_STENCIL_BIT,
            _ => {
                error!("Unexpected depth/stencil channel {:?}", channel);
                vk::IMAGE_ASPECT_DEPTH_BIT
            }
        },
        _ => vk::IMAGE_ASPECT_COLOR_BIT,
    }
}

pub fn map_channel_source(source: ChannelSource) -> vk::ComponentSwizzle {
    match source {
        ChannelSource::Zero => vk::COMPONENT_SWIZZLE_ZERO,
        ChannelSource::One  => vk::COMPONENT_SWIZZLE_ONE,
        ChannelSource::X    => vk::COMPONENT_SWIZZLE_R,
        ChannelSource::Y    => vk::COMPONENT_SWIZZLE_G,
        ChannelSource::Z    => vk::COMPONENT_SWIZZLE_B,
        ChannelSource::W    => vk::COMPONENT_SWIZZLE_A,
    }
}

pub fn map_swizzle(swizzle: Swizzle) -> vk::ComponentMapping {
    vk::ComponentMapping {
        r: map_channel_source(swizzle.0),
        g: map_channel_source(swizzle.1),
        b: map_channel_source(swizzle.2),
        a: map_channel_source(swizzle.3),
    }
}

pub fn map_usage_tiling(gfx_usage: Usage, bind: Bind) -> (vk::ImageUsageFlags, vk::ImageTiling) {
    use gfx_core::factory as f;
    let mut usage = 0;
    if bind.contains(f::RENDER_TARGET) {
        usage |= vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT;
    }
    if bind.contains(f::DEPTH_STENCIL) {
        usage |= vk::IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT;
    }
    if bind.contains(f::SHADER_RESOURCE) {
        usage |= vk::IMAGE_USAGE_SAMPLED_BIT;
    }
    if bind.contains(f::UNORDERED_ACCESS) {
        usage |= vk::IMAGE_USAGE_STORAGE_BIT;
    }
    let tiling = match gfx_usage {
        Usage::Const => vk::IMAGE_TILING_OPTIMAL,
        Usage::GpuOnly => {
            //TODO: not always needed
            usage |= vk::IMAGE_USAGE_TRANSFER_SRC_BIT | vk::IMAGE_USAGE_TRANSFER_DST_BIT;
            vk::IMAGE_TILING_OPTIMAL
        },
        Usage::Dynamic => {
            usage |= vk::IMAGE_USAGE_TRANSFER_DST_BIT;
            vk::IMAGE_TILING_LINEAR
        },
        Usage::CpuOnly(map) => {
            usage |= match map {
                MapAccess::Readable => vk::IMAGE_USAGE_TRANSFER_DST_BIT,
                MapAccess::Writable => vk::IMAGE_USAGE_TRANSFER_SRC_BIT,
                MapAccess::RW => vk::IMAGE_USAGE_TRANSFER_SRC_BIT | vk::IMAGE_USAGE_TRANSFER_DST_BIT,
            };
            vk::IMAGE_TILING_LINEAR
        },
    };
    (usage, tiling)
}

pub fn map_image_layout(bind: Bind) -> vk::ImageLayout {
    //use gfx_core::factory as f;
    // can't use optimal layouts for the fact PSO descriptor doesn't know about them
    match bind {
        //f::RENDER_TARGET   => vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
        //f::DEPTH_STENCIL   => vk::IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        //f::SHADER_RESOURCE => vk::IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
        _                  => vk::IMAGE_LAYOUT_GENERAL,
    }
}

pub fn map_format(surface: SurfaceType, chan: ChannelType) -> Option<vk::Format> {
    use gfx_core::format::SurfaceType::*;
    use gfx_core::format::ChannelType::*;
    Some(match surface {
        R4_G4 => match chan {
            Unorm => vk::FORMAT_R4G4_UNORM_PACK8,
            _ => return None,
        },
        R4_G4_B4_A4 => match chan {
            Unorm => vk::FORMAT_R4G4B4A4_UNORM_PACK16,
            _ => return None,
        },
        R5_G5_B5_A1 => match chan {
            Unorm => vk::FORMAT_R5G5B5A1_UNORM_PACK16,
             _ => return None,
        },
        R5_G6_B5 => match chan {
            Unorm => vk::FORMAT_R5G6B5_UNORM_PACK16,
             _ => return None,
        },
        R8 => match chan {
            Int   => vk::FORMAT_R8_SINT,
            Uint  => vk::FORMAT_R8_UINT,
            Inorm => vk::FORMAT_R8_SNORM,
            Unorm => vk::FORMAT_R8_UNORM,
            Srgb  => vk::FORMAT_R8_SRGB,
            _ => return None,
        },
        R8_G8 => match chan {
            Int   => vk::FORMAT_R8G8_SINT,
            Uint  => vk::FORMAT_R8G8_UINT,
            Inorm => vk::FORMAT_R8G8_SNORM,
            Unorm => vk::FORMAT_R8G8_UNORM,
            Srgb  => vk::FORMAT_R8G8_SRGB,
            _ => return None,
        },
        R8_G8_B8_A8 => match chan {
            Int   => vk::FORMAT_R8G8B8A8_SINT,
            Uint  => vk::FORMAT_R8G8B8A8_UINT,
            Inorm => vk::FORMAT_R8G8B8A8_SNORM,
            Unorm => vk::FORMAT_R8G8B8A8_UNORM,
            Srgb  => vk::FORMAT_R8G8B8A8_SRGB,
            _ => return None,
        },
        R10_G10_B10_A2 => match chan {
            Int   => vk::FORMAT_A2R10G10B10_SINT_PACK32,
            Uint  => vk::FORMAT_A2R10G10B10_UINT_PACK32,
            Inorm => vk::FORMAT_A2R10G10B10_SNORM_PACK32,
            Unorm => vk::FORMAT_A2R10G10B10_UNORM_PACK32,
            _ => return None,
        },
        R11_G11_B10 => match chan {
            Float => vk::FORMAT_B10G11R11_UFLOAT_PACK32,
            _ => return None,
        },
        R16 => match chan {
            Int   => vk::FORMAT_R16_SINT,
            Uint  => vk::FORMAT_R16_UINT,
            Inorm => vk::FORMAT_R16_SNORM,
            Unorm => vk::FORMAT_R16_UNORM,
            Float => vk::FORMAT_R16_SFLOAT,
            _ => return None,
        },
        R16_G16 => match chan {
            Int   => vk::FORMAT_R16G16_SINT,
            Uint  => vk::FORMAT_R16G16_UINT,
            Inorm => vk::FORMAT_R16G16_SNORM,
            Unorm => vk::FORMAT_R16G16_UNORM,
            Float => vk::FORMAT_R16G16_SFLOAT,
            _ => return None,
        },
        R16_G16_B16 => match chan {
            Int   => vk::FORMAT_R16G16B16_SINT,
            Uint  => vk::FORMAT_R16G16B16_UINT,
            Inorm => vk::FORMAT_R16G16B16_SNORM,
            Unorm => vk::FORMAT_R16G16B16_UNORM,
            Float => vk::FORMAT_R16G16B16_SFLOAT,
            _ => return None,
        },
        R16_G16_B16_A16 => match chan {
            Int   => vk::FORMAT_R16G16B16A16_SINT,
            Uint  => vk::FORMAT_R16G16B16A16_UINT,
            Inorm => vk::FORMAT_R16G16B16A16_SNORM,
            Unorm => vk::FORMAT_R16G16B16A16_UNORM,
            Float => vk::FORMAT_R16G16B16A16_SFLOAT,
            _ => return None,
        },
        R32 => match chan {
            Int   => vk::FORMAT_R32_SINT,
            Uint  => vk::FORMAT_R32_UINT,
            Float => vk::FORMAT_R32_SFLOAT,
            _ => return None,
        },
        R32_G32 => match chan {
            Int   => vk::FORMAT_R32G32_SINT,
            Uint  => vk::FORMAT_R32G32_UINT,
            Float => vk::FORMAT_R32G32_SFLOAT,
            _ => return None,
        },
        R32_G32_B32 => match chan {
            Int   => vk::FORMAT_R32G32B32_SINT,
            Uint  => vk::FORMAT_R32G32B32_UINT,
            Float => vk::FORMAT_R32G32B32_SFLOAT,
            _ => return None,
        },
        R32_G32_B32_A32 => match chan {
            Int   => vk::FORMAT_R32G32B32A32_SINT,
            Uint  => vk::FORMAT_R32G32B32A32_UINT,
            Float => vk::FORMAT_R32G32B32A32_SFLOAT,
            _ => return None,
        },
        D16 => match chan {
            Unorm  => vk::FORMAT_D16_UNORM,
            _ => return None,
        },
        D24 => match chan {
            Unorm => vk::FORMAT_X8_D24_UNORM_PACK32,
            _ => return None,
        },
        D24_S8 => match chan {
            Unorm => vk::FORMAT_D24_UNORM_S8_UINT,
            _ => return None,
        },
        D32 => match chan {
            Float => vk::FORMAT_D32_SFLOAT,
            _ => return None,
        },
    })
}

pub fn map_filter(filter: FilterMethod) -> (vk::Filter, vk::Filter, vk::SamplerMipmapMode, f32) {
    match filter {
        FilterMethod::Scale          => (vk::FILTER_NEAREST, vk::FILTER_NEAREST, vk::SAMPLER_MIPMAP_MODE_NEAREST, 0.0),
        FilterMethod::Mipmap         => (vk::FILTER_NEAREST, vk::FILTER_NEAREST, vk::SAMPLER_MIPMAP_MODE_LINEAR,  0.0),
        FilterMethod::Bilinear       => (vk::FILTER_LINEAR,  vk::FILTER_LINEAR,  vk::SAMPLER_MIPMAP_MODE_NEAREST, 0.0),
        FilterMethod::Trilinear      => (vk::FILTER_LINEAR,  vk::FILTER_LINEAR,  vk::SAMPLER_MIPMAP_MODE_LINEAR,  0.0),
        FilterMethod::Anisotropic(a) => (vk::FILTER_LINEAR,  vk::FILTER_LINEAR,  vk::SAMPLER_MIPMAP_MODE_LINEAR,  a as f32),
    }
}

pub fn map_wrap(wrap: WrapMode) -> vk::SamplerAddressMode {
    match wrap {
        WrapMode::Tile   => vk::SAMPLER_ADDRESS_MODE_REPEAT,
        WrapMode::Mirror => vk::SAMPLER_ADDRESS_MODE_MIRRORED_REPEAT,
        WrapMode::Clamp  => vk::SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
        WrapMode::Border => vk::SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER,
    }
}

pub fn map_border_color(col: PackedColor) -> Option<vk::BorderColor> {
    match col.0 {
        0x00000000 => Some(vk::BORDER_COLOR_FLOAT_TRANSPARENT_BLACK),
        0xFF000000 => Some(vk::BORDER_COLOR_FLOAT_OPAQUE_BLACK),
        0xFFFFFFFF => Some(vk::BORDER_COLOR_FLOAT_OPAQUE_WHITE),
        _ => None
    }
}

pub fn map_comparison(fun: state::Comparison) -> vk::CompareOp {
    use gfx_core::state::Comparison::*;
    match fun {
        Never        => vk::COMPARE_OP_NEVER,
        Less         => vk::COMPARE_OP_LESS,
        LessEqual    => vk::COMPARE_OP_LESS_OR_EQUAL,
        Equal        => vk::COMPARE_OP_EQUAL,
        GreaterEqual => vk::COMPARE_OP_GREATER_OR_EQUAL,
        Greater      => vk::COMPARE_OP_GREATER,
        NotEqual     => vk::COMPARE_OP_NOT_EQUAL,
        Always       => vk::COMPARE_OP_ALWAYS,
    }
}
