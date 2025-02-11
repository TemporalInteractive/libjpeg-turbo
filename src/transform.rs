use crate::ffi;

use crate::buf::{OutputBuf, OwnedBuf};
use crate::common::{Error, Result};
use crate::handle::Handle;
use std::convert::TryInto as _;
use std::ptr;

/// Transforms JPEG images without recompression.
///
/// TurboJPEG applies the transformation on the DCT coefficients, without performing complete
/// decompression. This is faster and also means that the transforms are lossless.
#[derive(Debug)]
#[doc(alias = "tjhandle")]
pub struct Transformer {
    handle: Handle,
}

/// Lossless transform of a JPEG image.
///
/// When constructing an instance, you may start from the default transform
/// ([`Transform::default()`][Self::default]) or from an operation ([`Transform::op()`]) and modify
/// only the fields that you need.
///
/// # Examples
///
/// Rotate image clockwise by 90 degrees:
///
/// ```
/// # use turbojpeg::{Transform, TransformOp};
/// let transform = Transform::op(TransformOp::Rot90);
/// ```
///
/// Rotate image counterclockwise by 90 degrees and fail if the transform is not
/// [perfect][Self::perfect]:
///
/// ```
/// # use turbojpeg::{Transform, TransformOp};
/// let mut transform = Transform::op(TransformOp::Rot270);
/// transform.perfect = true;
/// ```
///
/// Flip image vertically and [trim][Self::trim] the image on the right edge if the transform is
/// imperfect:
///
/// ```
/// # use turbojpeg::{Transform, TransformOp};
/// let mut transform = Transform::op(TransformOp::Vflip);
/// transform.trim = true;
/// ```
///
/// Crop image to size (200, 100) starting at pixel (16, 32), without applying any transform:
///
/// ```
/// # use turbojpeg::{Transform, TransformOp, TransformCrop};
/// let mut transform = Transform::default();
/// transform.crop = Some(TransformCrop { x: 16, y: 32, width: Some(200), height: Some(100) });
/// ```
#[derive(Debug, Default, Clone)]
#[doc(alias = "tjtransform")]
#[non_exhaustive]
pub struct Transform {
    /// Transform operation that is applied.
    pub op: TransformOp,

    /// Crop the input image before applying the transform.
    #[doc(alias = "TJXOPT_CROP")]
    pub crop: Option<TransformCrop>,

    /// Return an error if the transform is not perfect.
    ///
    /// Lossless transforms operate on MCU blocks, whose size depends on the level of chrominance
    /// subsampling used (see [`Subsamp::mcu_width()`][crate::Subsamp::mcu_width] and
    /// [`Subsamp::mcu_height()`][crate::Subsamp::mcu_height]). If the image width or height is not
    /// evenly divisible by the MCU block size, then there will be partial MCU blocks on the right
    /// and bottom edges. It is not possible to move these partial MCU blocks to the top or left of
    /// the image, so any transform that would require that is "imperfect".
    ///
    /// If this option is not specified and [`trim`][Self::trim] is not enabled, then any partial
    /// MCU blocks that cannot be transformed will be left in place, which will create odd-looking
    /// strips on the right or bottom edge of the image.
    #[doc(alias = "TJXOPT_PERFECT")]
    pub perfect: bool,

    /// Discard any partial MCU blocks that cannot be transformed.
    #[doc(alias = "TJXOPT_TRIM")]
    pub trim: bool,

    /// Discard the color data in the input image and produce a grayscale output image.
    #[doc(alias = "TJXOPT_GRAY")]
    pub gray: bool,

    /// Enable progressive entropy coding in the output image generated by this particular
    /// transform.
    ///
    /// Progressive entropy coding will generally improve compression relative to baseline entropy
    /// coding (the default), but it will reduce compression and decompression performance
    /// considerably.
    #[doc(alias = "TJXOPT_PROGRESSIVE")]
    pub progressive: bool,

    /// Enable optimized baseline entropy coding in the JPEG image generated by this particular
    /// transform.
    ///
    /// Optimized baseline entropy coding will improve compression slightly (generally 5% or less.)
    #[doc(alias = "TJXOPT_OPTIMIZE")]
    pub optimize: bool,

    /// Do not copy any extra markers (including EXIF and ICC profile data) from the input image to
    /// the output image.
    #[doc(alias = "TJXOPT_COPYNONE")]
    pub copy_none: bool,
}

impl Transform {
    /// Creates a [`Transform`] with the given `op` and all other parameters set to default.
    ///
    /// # Example
    //
    /// ```
    /// # use turbojpeg::{Transform, TransformOp};
    /// let mut transform = Transform::op(TransformOp::Rot90);
    /// transform.progressive = true;
    /// ```
    pub fn op(op: TransformOp) -> Transform {
        Transform {
            op,
            ..Transform::default()
        }
    }
}

/// Transform operation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "TJXOP")]
#[repr(u32)]
#[non_exhaustive]
pub enum TransformOp {
    /// No transformation (noop).
    #[doc(alias = "TJXOP_NONE")]
    None = ffi::TJXOP_TJXOP_NONE as u32,

    /// Flip (mirror) image horizontally.
    ///
    /// This transform is imperfect if there are any partial MCU blocks on the right edge (see
    /// [`Transform::perfect`].)
    #[doc(alias = "TJXOP_HFLIP")]
    Hflip = ffi::TJXOP_TJXOP_HFLIP as u32,

    /// Flip (mirror) image vertically.
    ///
    /// This transform is imperfect if there are any partial MCU blocks on the bottom edge (see
    /// [`Transform::perfect`].)
    #[doc(alias = "TJXOP_VFLIP")]
    Vflip = ffi::TJXOP_TJXOP_VFLIP as u32,

    /// Transpose image (flip/mirror along upper left to lower right axis).
    ///
    /// This transform is always perfect.
    #[doc(alias = "TJXOP_TRANSPOSE")]
    Transpose = ffi::TJXOP_TJXOP_TRANSPOSE as u32,

    /// Transverse transpose image (flip/mirror along upper right to lower left axis).
    ///
    /// This transform is imperfect if there are any partial MCU blocks in the image (see
    /// [`Transform::perfect`].)
    #[doc(alias = "TJXOP_TRANSVERSE")]
    Transverse = ffi::TJXOP_TJXOP_TRANSVERSE as u32,

    /// Rotate image clockwise by 90 degrees.
    ///
    /// This transform is imperfect if there are any partial MCU blocks on the bottom edge (see
    /// [`Transform::perfect`].)
    #[doc(alias = "TJXOP_ROT90")]
    Rot90 = ffi::TJXOP_TJXOP_ROT90 as u32,

    /// Rotate image 180 degrees.
    ///
    /// This transform is imperfect if there are any partial MCU blocks in the image (see
    /// [`Transform::perfect`].)
    #[doc(alias = "TJXOP_ROT180")]
    Rot180 = ffi::TJXOP_TJXOP_ROT180 as u32,

    /// Rotate image counter-clockwise by 90 degrees.
    ///
    /// This transform is imperfect if there are any partial MCU blocks on the right edge (see
    /// [`Transform::perfect`].)
    Rot270 = ffi::TJXOP_TJXOP_ROT270 as u32,
}

impl Default for TransformOp {
    fn default() -> Self {
        TransformOp::None
    }
}

/// Transform cropping region.
///
/// The [`x`][Self::x] and [`y`][Self::y] position of the region must be aligned on MCU boundaries.
/// The size of the MCU depends on the chrominance subsampling option, which can be obtained using
/// [`Decompressor::read_header()`][crate::Decompressor::read_header].
///
/// The default instance performs no cropping.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias = "tjregion")]
pub struct TransformCrop {
    /// Left boundary of the region. This must be divisible by the MCU width (see
    /// [`Subsamp::mcu_width()`][crate::Subsamp::mcu_width]).
    pub x: usize,
    /// Upper boundary of the region. This must be divisible by the MCU height (see
    /// [`Subsamp::mcu_height()`][crate::Subsamp::mcu_height]).
    pub y: usize,
    /// Width of the region. If None is given, the region ends at the right boundary of the image.
    pub width: Option<usize>,
    /// Height of the region. If None is given, the region ends at the bottom boundary of the
    /// image.
    pub height: Option<usize>,
}

impl Transformer {
    /// Create a new transformer instance.
    #[doc(alias = "tj3Init")]
    pub fn new() -> Result<Transformer> {
        let handle = Handle::new(ffi::TJINIT_TJINIT_TRANSFORM)?;
        Ok(Self { handle })
    }

    /// Apply a transformation to the compressed JPEG.
    ///
    /// This is the main transformation method, which gives you full control of the output buffer. If
    /// you don't need this level of control, you can use one of the convenience wrappers below.
    ///
    /// # Example
    ///
    /// ```
    /// // read JPEG data from file
    /// let jpeg_data = std::fs::read("examples/parrots.jpg")?;
    ///
    /// // initialize the transformer
    /// let mut transformer = turbojpeg::Transformer::new()?;
    ///
    /// // define the transformation: flip vertically, trim partial MCU blocks on the bottom edge
    /// let mut transform = turbojpeg::Transform::op(turbojpeg::TransformOp::Vflip);
    /// transform.trim = true;
    ///
    /// // initialize the output buffer
    /// let mut flipped_data = turbojpeg::OutputBuf::new_owned();
    ///
    /// // apply the transformation
    /// transformer.transform(&transform, &jpeg_data, &mut flipped_data)?;
    ///
    /// // write the flipped JPEG back to disk
    /// std::fs::write(std::env::temp_dir().join("flipped_parrots.jpg"), &flipped_data)?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[doc(alias = "tj3Transform")]
    pub fn transform(
        &mut self,
        transform: &Transform,
        jpeg_data: &[u8],
        output: &mut OutputBuf,
    ) -> Result<()> {
        let mut options = 0;
        if transform.perfect {
            options |= ffi::TJXOPT_PERFECT
        }
        if transform.trim {
            options |= ffi::TJXOPT_TRIM
        }
        if transform.gray {
            options |= ffi::TJXOPT_GRAY
        }
        if transform.progressive {
            options |= ffi::TJXOPT_PROGRESSIVE
        }
        if transform.optimize {
            options |= ffi::TJXOPT_OPTIMIZE
        }
        if transform.copy_none {
            options |= ffi::TJXOPT_COPYNONE
        }

        let mut region = ffi::tjregion {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        };
        if let Some(crop) = transform.crop {
            region.x = crop
                .x
                .try_into()
                .map_err(|_| Error::IntegerOverflow("crop.x"))?;
            region.y = crop
                .y
                .try_into()
                .map_err(|_| Error::IntegerOverflow("crop.y"))?;
            if let Some(crop_w) = crop.width {
                region.w = crop_w
                    .try_into()
                    .map_err(|_| Error::IntegerOverflow("crop.width"))?;
            }
            if let Some(crop_h) = crop.height {
                region.h = crop_h
                    .try_into()
                    .map_err(|_| Error::IntegerOverflow("crop.height"))?;
            }
            options |= ffi::TJXOPT_CROP;
        }

        let mut transform = ffi::tjtransform {
            r: region,
            op: transform.op as libc::c_int,
            options: options as libc::c_int,
            data: ptr::null_mut(),
            customFilter: None,
        };

        self.handle.set(
            ffi::TJPARAM_TJPARAM_NOREALLOC,
            if output.is_owned { 0 } else { 1 } as libc::c_int,
        )?;
        let mut output_len = output.len as ffi::size_t;
        let res = unsafe {
            ffi::tj3Transform(
                self.handle.as_ptr(),
                jpeg_data.as_ptr(),
                jpeg_data.len() as ffi::size_t,
                1,
                &mut output.ptr,
                &mut output_len,
                &mut transform,
            )
        };
        output.len = output_len as usize;
        if res != 0 {
            return Err(self.handle.get_error());
        } else if output.ptr.is_null() {
            output.len = 0;
            return Err(Error::Null);
        }

        Ok(())
    }

    /// Transforms the `image` into an owned buffer.
    ///
    /// This method automatically allocates the memory and avoids needless copying.
    pub fn transform_to_owned(
        &mut self,
        transform: &Transform,
        jpeg_data: &[u8],
    ) -> Result<OwnedBuf> {
        let mut buf = OutputBuf::new_owned();
        self.transform(transform, jpeg_data, &mut buf)?;
        Ok(buf.into_owned())
    }

    /// Transform the `image` into a new `Vec<u8>`.
    ///
    /// This method copies the transformed data into a new `Vec`. If you would like to avoid the
    /// extra allocation and copying, consider using
    /// [`transform_to_owned()`][Self::transform_to_owned] instead.
    pub fn transform_to_vec(&mut self, transform: &Transform, jpeg_data: &[u8]) -> Result<Vec<u8>> {
        let mut buf = OutputBuf::new_owned();
        self.transform(transform, jpeg_data, &mut buf)?;
        Ok(buf.to_vec())
    }

    /// Transform the `image` into the slice `output`.
    ///
    /// Returns the size of the transformed JPEG data. If the transformed image does not fit into
    /// `dest`, this method returns an error.
    ///
    /// You can use [`compressed_buf_len()`][crate::compressed_buf_len] to determine buffer size that
    /// should be enough for the image, but there are some rare cases (such as transforming images
    /// with a large amount of embedded EXIF or ICC profile data) in which the output image will be
    /// larger than the size returned by [`compressed_buf_len()`][crate::compressed_buf_len].
    pub fn transform_to_slice(
        &mut self,
        transform: &Transform,
        jpeg_data: &[u8],
        output: &mut [u8],
    ) -> Result<usize> {
        let mut buf = OutputBuf::borrowed(output);
        self.transform(transform, jpeg_data, &mut buf)?;
        Ok(buf.len())
    }
}

/// Losslessly transform a JPEG image without recompression.
///
/// TurboJPEG applies the transformation on the DCT coefficients, without performing complete
/// decompression. This is faster and also means that the transforms are lossless.
///
/// Returns the transformed JPEG data in a buffer owned by TurboJPEG. If this does not fit your
/// needs, please see [`Transformer`].
///
/// # Example
///
/// ```
/// // read JPEG data from file
/// let jpeg_data = std::fs::read("examples/parrots.jpg")?;
///
/// // define the transformation: rotate 90 degrees clockwise
/// let mut transform = turbojpeg::Transform::op(turbojpeg::TransformOp::Rot90);
/// transform.optimize = true;
///
/// // apply the transformation
/// let rotated_data = turbojpeg::transform(&transform, &jpeg_data)?;
///
/// // write the rotated JPEG back to disk
/// std::fs::write(std::env::temp_dir().join("rotated_parrots.jpg"), &rotated_data)?;
///
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn transform(transform: &Transform, jpeg_data: &[u8]) -> Result<OwnedBuf> {
    let mut transformer = Transformer::new()?;
    transformer.transform_to_owned(transform, jpeg_data)
}
