use crate::{
    n2::point::p2,
    n3::shape::Ray,
    nbase::{bounds::Bounds, line_segment::LineSegment, lineset::LineSet, point::Point},
};

use self::shape::Shape;

use async_trait::async_trait;
use nalgebra as na;
use tokio::sync::mpsc::Sender;

pub mod shape;

pub struct Scene<'a> {
    shapes: Vec<Box<dyn Shape + 'a>>,
}

pub struct CullingInfo {}

pub fn create_culling_info(camera: &Camera) -> CullingInfo {
    CullingInfo {}
}

pub struct OcclusionInfo<'a, 'b> {
    eye: Point<3>,
    shapes: &'a [Box<dyn Shape + 'b>],
}

impl<'a, 'b> OcclusionInfo<'a, 'b> {
    fn is_occluded(&self, p: &Point<3>) -> bool {
        let ray = Ray {
            origin: *p,
            direction: self.eye - *p,
        };
        for s in self.shapes {
            if s.intersect(&ray).is_some() {
                return true;
            }
        }
        false
    }
}

pub fn create_occlusion_info<'a, 'b>(
    camera: &Camera,
    shapes: &'a [Box<dyn Shape + 'b>],
) -> OcclusionInfo<'a, 'b> {
    OcclusionInfo {
        eye: camera.eye,
        shapes,
    }
}

impl<'a> Scene<'a> {
    pub fn new() -> Scene<'a> {
        Scene { shapes: vec![] }
    }

    pub fn add_primitive<T>(&mut self, shape: T)
    where
        T: 'static + Shape,
    {
        self.shapes.push(Box::new(shape));
    }

    pub async fn render(
        &self,
        camera: &Camera,
        consumer: Sender<LineSegment<2, ()>>,
    ) -> Result<(), std::io::Error> {
        let culling_info = create_culling_info(camera);
        let occlusion_info = create_occlusion_info(camera, &self.shapes);
        for s in &self.shapes {
            s.render(camera, &occlusion_info, consumer.clone()).await?;
        }
        Ok(())
    }
}

pub fn p3(x: f32, y: f32, z: f32) -> Point<3> {
    Point::from([x, y, z])
}

pub struct Camera {
    canvas: Bounds<2>,
    eye: Point<3>,
    fwd: Point<3>,
    right: Point<3>,
    up: Point<3>,

    //TODO: Probably doesn't belong here
    // Maximum length of line-segments.
    // Larger ones will be split
    lmax: f32,
    lmin: f32,
}
impl Camera {
    #[allow(clippy::many_single_char_names)]
    fn to_camera_coordinates(&self, p: &Point<3>) -> Point<3> {
        // Transform to camera coordinates.
        // TODO: Store some of this in the camera object.
        let eye = na::Vector3::from(self.eye.vs);
        let fwd = na::Vector3::from(self.fwd.vs);
        let right = na::Vector3::from(self.right.vs);
        let up = na::Vector3::from(self.up.vs);

        // We trust the fwd vector the most.
        // up should be orthogonal to fwd.
        // but it might not be, so to fix that we add a small amount
        // of fwd.
        // u' = u - a f
        // 0=<u',f> = <u,f> - a <f,f>
        // a = <u,f>/<f,f>
        let f = fwd;
        let a = up.dot(&f) / f.dot(&f);
        let u = up - a * f;

        // eprintln!("f={:?}", f);
        // eprintln!("up={:?} u={:?}",up, u);

        // Now we don't trust right. It should be orthogonal to f and u
        // so we adjust it by adding some f and some u
        // r' = r + b f + c u'
        // 0 = <r',f> = <r,f> - b <f,f> - c<f,u'> => b=<r,f>/<f,f>
        // 0 = <r',u'> = <r,u'> -b <f,u'> -c<u',u'> => c = <r,u'>/<u',u'>
        let b = right.dot(&f) / f.dot(&f);
        let c = right.dot(&u) / u.dot(&u);
        let r = right - b * f - c * u;
        // eprintln!("right={:?} r={:?}",right, r);

        let m = na::Matrix3::from_columns(&[r, u, f]);
        let invM = m.try_inverse().unwrap();
        let p = na::Vector3::from(p.vs);

        let d = p - eye;
        let dd = invM * d;
        Point::from([dd[0] / dd[2], dd[1] / dd[2], dd[2]])
    }

    fn project_pt(&self, p: &Point<3>) -> Point<2> {
        let p = self.to_camera_coordinates(p);
        let m = (self.canvas.min + self.canvas.max) * 0.5;
        let d = (self.canvas.max - self.canvas.min) * 0.5;
        m + Point::from([p.vs[0], -p.vs[1]]) * d
    }

    fn is_segment_visible(&self, ls: &LineSegment<3, ()>) -> bool {
        let p1 = self.to_camera_coordinates(&ls.ps[0]);
        let p2 = self.to_camera_coordinates(&ls.ps[1]);

        // At least one has to be in front of the z=1 clipping plane
        if p1.vs[2] < 1.0 && p2.vs[2] < 1.0 {
            return false;
        }

        // If either end is inside the view then the segment is
        // visible
        if -1.0 <= p1.vs[0] && p1.vs[0] <= 1.0 && -1.0 <= p1.vs[1] && p1.vs[1] <= 1.0 {
            return true;
        }

        if -1.0 <= p2.vs[0] && p2.vs[0] <= 1.0 && -1.0 <= p2.vs[1] && p2.vs[1] <= 1.0 {
            return true;
        }

        // Both points fall in one of the excluding half planes => false
        if p1.vs[0] < -1.0 && p2.vs[0] < -1.0 {
            return false;
        }

        if p1.vs[0] > 1.0 && p2.vs[0] > 1.0 {
            return false;
        }

        if p1.vs[1] < -1.0 && p2.vs[1] < -1.0 {
            return false;
        }

        if p1.vs[1] > 1.0 && p2.vs[1] > 1.0 {
            return false;
        }

        // We could be cleverer here, but we don't need to be exact.
        // So we'll just accept everything else.
        true
    }

    fn project(&self, ls: &LineSegment<3, ()>) -> LineSegment<2, ()> {
        LineSegment {
            ps: ls.ps.map(|p| self.project_pt(&p)),
            attributes: (),
        }
    }
}

#[derive(Debug)]
pub enum CameraBuilderError {
    MissingFields(Vec<String>),
}

#[derive(Default, Debug, Clone)]
pub struct CameraBuilder {
    canvas: Option<Bounds<2>>,
    eye: Option<Point<3>>,
    fwd: Option<Point<3>>,
    right: Option<Point<3>>,
    up: Option<Point<3>>,
}

impl CameraBuilder {
    pub fn builder() -> CameraBuilder {
        CameraBuilder::default()
    }

    pub fn canvas(&self, xmin: f32, ymin: f32, xmax: f32, ymax: f32) -> CameraBuilder {
        let mut result = self.clone();
        result.canvas = Some(Bounds {
            min: p2(xmin, ymin),
            max: p2(xmax, ymax),
        });
        result
    }

    pub fn eye(&self, x: f32, y: f32, z: f32) -> CameraBuilder {
        let mut result = self.clone();
        result.eye = Some(p3(x, y, z));
        result
    }

    pub fn fwd(&self, x: f32, y: f32, z: f32) -> CameraBuilder {
        let mut result = self.clone();
        result.fwd = Some(p3(x, y, z));
        result
    }

    pub fn right(&self, x: f32, y: f32, z: f32) -> CameraBuilder {
        let mut result = self.clone();
        result.right = Some(p3(x, y, z));
        result
    }

    pub fn up(&self, x: f32, y: f32, z: f32) -> CameraBuilder {
        let mut result = self.clone();
        result.up = Some(p3(x, y, z));
        result
    }

    pub fn create(&self) -> Result<Camera, CameraBuilderError> {
        let mut missing_fields = vec![];
        if self.canvas.is_none() {
            missing_fields.push("canvas".to_string());
        }
        if self.eye.is_none() {
            missing_fields.push("eye".to_string());
        }
        if self.fwd.is_none() {
            missing_fields.push("fwd".to_string());
        }
        if self.right.is_none() {
            missing_fields.push("right".to_string());
        }
        if self.up.is_none() {
            missing_fields.push("up".to_string());
        }
        if !missing_fields.is_empty() {
            return Err(CameraBuilderError::MissingFields(missing_fields));
        }
        Ok(Camera {
            canvas: self.canvas.unwrap(),
            eye: self.eye.unwrap(),
            fwd: self.fwd.unwrap(),
            right: self.right.unwrap(),
            up: self.up.unwrap(),
            lmax: 2.0,
            lmin: 0.2,
        })
    }
}

pub trait Texture {
    fn apply(&self, p: Point<2>) -> f32;
}

// How does this differ from Shading in n2::hl and can we combine them?

#[async_trait]
pub trait ScreenSpaceTexture {
    async fn apply(
        &self,
        screen_bounds: Bounds<2>,
        brightness: &dyn Texture,
        mask: &dyn Texture,
        consumer: Sender<LineSegment<2, ()>>,
    );
}

#[async_trait]
pub trait ObjectSpaceTexture {
    async fn apply(
        &self,
        uv_bounds: Bounds<2>,
        uv_brightness: &dyn Texture,
        uv_mask: &dyn Texture,
        consumer: Sender<LineSegment<3, ()>>,
    );
}
