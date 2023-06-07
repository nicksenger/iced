use iced_core::Point;
use lyon::algorithms::measure::PathMeasurements;
use lyon::geom::point;
use lyon::path::builder::NoAttributes;
use lyon::path::path::BuilderImpl;
use lyon::path::Path;

use once_cell::sync::Lazy;

pub static EMPHASIZED: Lazy<Easing> = Lazy::new(|| {
    Easing::builder()
        .cubic_bezier_to([0.05, 0.0], [0.133333, 0.06], [0.166666, 0.4])
        .cubic_bezier_to([0.208333, 0.82], [0.25, 1.0], [1.0, 1.0])
        .build()
});

pub static EMPHASIZED_DECELERATE: Lazy<Easing> = Lazy::new(|| {
    Easing::builder()
        .cubic_bezier_to([0.05, 0.7], [0.1, 1.0], [1.0, 1.0])
        .build()
});

pub static EMPHASIZED_ACCELERATE: Lazy<Easing> = Lazy::new(|| {
    Easing::builder()
        .cubic_bezier_to([0.3, 0.0], [0.8, 0.15], [1.0, 1.0])
        .build()
});

pub static STANDARD: Lazy<Easing> = Lazy::new(|| {
    Easing::builder()
        .cubic_bezier_to([0.2, 0.0], [0.0, 1.0], [1.0, 1.0])
        .build()
});

pub static STANDARD_DECELERATE: Lazy<Easing> = Lazy::new(|| {
    Easing::builder()
        .cubic_bezier_to([0.0, 0.0], [0.0, 1.0], [1.0, 1.0])
        .build()
});

pub static STANDARD_ACCELERATE: Lazy<Easing> = Lazy::new(|| {
    Easing::builder()
        .cubic_bezier_to([0.3, 0.0], [1.0, 1.0], [1.0, 1.0])
        .build()
});

pub struct Easing {
    path: Path,
    measurements: PathMeasurements,
}

impl Easing {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn y_at_x(&self, x: f32) -> f32 {
        let mut sampler = self.measurements.create_sampler(
            &self.path,
            lyon::algorithms::measure::SampleType::Normalized,
        );
        let sample = sampler.sample(x);

        sample.position().y
    }
}

pub struct Builder(NoAttributes<BuilderImpl>);

impl Builder {
    pub fn new() -> Self {
        let mut builder = Path::builder();
        builder.begin(point(0.0, 0.0));

        Self(builder)
    }

    /// Adds a line segment.
    pub fn line_to(mut self, to: impl Into<Point>) -> Self {
        let to: Point = to.into();
        self.0.line_to(point(to.x, to.y));

        self
    }

    /// Adds a quadratic bézier curve.
    pub fn quadratic_bezier_to(
        mut self,
        ctrl: impl Into<Point>,
        to: impl Into<Point>,
    ) -> Self {
        let [c, p]: [Point; 2] = [ctrl.into(), to.into()];
        self.0.quadratic_bezier_to(point(c.x, c.y), point(p.x, p.y));

        self
    }

    /// Adds a cubic bézier curve.
    pub fn cubic_bezier_to(
        mut self,
        ctrl1: impl Into<Point>,
        ctrl2: impl Into<Point>,
        to: impl Into<Point>,
    ) -> Self {
        let [c1, c2, p]: [Point; 3] = [ctrl1.into(), ctrl2.into(), to.into()];
        self.0.cubic_bezier_to(
            point(c1.x, c1.y),
            point(c2.x, c2.y),
            point(p.x, p.y),
        );

        self
    }

    pub fn build(mut self) -> Easing {
        self.0.line_to(point(1.0, 1.0));
        self.0.end(false);

        let path = self.0.build();
        let measurements = PathMeasurements::from_path(&path, 0.0);

        Easing { path, measurements }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
