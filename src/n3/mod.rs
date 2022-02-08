use crate::{
    n2::point::p2,
    nbase::{bounds::Bounds, point::Point, polyline::LineSegment},
};

use self::shape::{Consumer, Shape};

pub mod shape;

pub struct Scene<'a> {
    shapes: Vec<Box<dyn Shape + 'a>>,
}

pub struct CullingInfo {}

pub fn create_culling_info(camera: &Camera) -> CullingInfo {
    CullingInfo {}
}

pub struct OcclusionInfo {}

pub fn create_occlusion_info(camera: &Camera, shapes: &[Box<dyn Shape + '_>]) -> OcclusionInfo {
    OcclusionInfo {}
}

impl<'a> Scene<'a> {
    pub(crate) fn new() -> Scene<'a> {
        Scene { shapes: vec![] }
    }

    pub(crate) fn add_primitive<T>(&mut self, shape: T)
    where
        T: 'static + Shape,
    {
        self.shapes.push(Box::new(shape));
    }

    pub(crate) fn render(
        &self,
        camera: &Camera,
        consumer: &mut dyn Consumer,
    ) -> Result<(), std::io::Error> {
        let culling_info = create_culling_info(camera);
        let occlusion_info = create_occlusion_info(camera, &self.shapes);
        for s in &self.shapes {
            s.render(camera, &occlusion_info, consumer)?;
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
}
impl Camera {
    fn is_segment_visible(&self, ls: &LineSegment<3>) -> bool {
        // Transform to camera coordinates.

        true
    }

    fn project_pt(&self, p: &Point<3>) -> Point<2> {
        Point {
            vs: [p.vs[0], p.vs[1]],
        }
    }

    fn project(&self, ls: &LineSegment<3>) -> LineSegment<2> {
        LineSegment {
            ps: ls.ps.map(|p| self.project_pt(&p)),
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
    pub(crate) fn builder() -> CameraBuilder {
        CameraBuilder::default()
    }

    pub(crate) fn canvas(&self, xmin: f32, ymin: f32, xmax: f32, ymax: f32) -> CameraBuilder {
        let mut result = self.clone();
        result.canvas = Some(Bounds {
            min: p2(xmin, ymin),
            max: p2(xmax, ymax),
        });
        result
    }

    pub(crate) fn eye(&self, x: f32, y: f32, z: f32) -> CameraBuilder {
        let mut result = self.clone();
        result.eye = Some(p3(x, y, z));
        result
    }

    pub(crate) fn fwd(&self, x: f32, y: f32, z: f32) -> CameraBuilder {
        let mut result = self.clone();
        result.fwd = Some(p3(x, y, z));
        result
    }

    pub(crate) fn right(&self, x: f32, y: f32, z: f32) -> CameraBuilder {
        let mut result = self.clone();
        result.right = Some(p3(x, y, z));
        result
    }

    pub(crate) fn up(&self, x: f32, y: f32, z: f32) -> CameraBuilder {
        let mut result = self.clone();
        result.up = Some(p3(x, y, z));
        result
    }

    pub(crate) fn create(&self) -> Result<Camera, CameraBuilderError> {
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
            lmax: 1.0,
        })
    }
}
