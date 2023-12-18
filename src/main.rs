use std::iter::once;

use itertools::Itertools;
use nalgebra::{Isometry, Isometry2, Point2, Vector2};
use ncollide2d::bounding_volume::{BoundingSphere, BoundingVolume};
use ncollide2d::query::PointQuery;
use ncollide2d::shape::{Ball, ConvexPolygon, ConvexPolyhedron, Segment, Shape, ShapeHandle};
use ordered_float::OrderedFloat;
use voronoi::voronoi;

fn incircle(polygon: &ConvexPolygon<f64>) -> BoundingSphere<f64> {
    let voronoi_diagram = voronoi(
        polygon
            .points()
            .iter()
            .map(|p| voronoi::Point::new(p.x, p.y))
            .collect::<Vec<_>>(),
        1.0,
    );

    let mut max_distance = 0.0;
    let mut center = Point2::origin();

    let points = voronoi_diagram
        .vertices
        .iter()
        .map(|node| Point2::new(node.coordinates.x(), node.coordinates.y()))
        .filter(|point| polygon.contains_point(&Isometry::identity(), point));

    let max_distance = points
        .map(|point| {
            let distances_to_pairs = polygon
                .points()
                .iter()
                .chain(once(polygon.points().first().unwrap()))
                .cloned()
                .tuple_windows()
                .map(|(prev, next)| Segment::new(prev, next))
                .map(|segment: Segment<f64>| {
                    OrderedFloat(segment.distance_to_point(&Isometry::identity(), &point, true))
                });
            distances_to_pairs.min().unwrap_or(OrderedFloat(f64::MAX))
        })
        .max()
        .unwrap_or_default();
    BoundingSphere::new(center, max_distance.0)
}

fn circumcircle(polygon: &ConvexPolygon<f64>) -> BoundingSphere<f64> {
    polygon.bounding_sphere(&Isometry2::identity())
}

fn roundness(polygon: &ConvexPolygon<f64>) -> f64 {
    let circumcircle = circumcircle(polygon);
    let incircle = incircle(polygon);
    dbg!(circumcircle);
    dbg!(incircle);
    (incircle.radius() / circumcircle.radius()).powi(2)
}

fn main() {
    let points = [
        Point2::new(0.0, 0.0),
        Point2::new(0.0, 1.0),
        Point2::new(1.0, 0.0),
        Point2::new(1.0, 1.0),
    ];
    let polygon = ConvexPolygon::try_from_points(&points).unwrap();
    let current_roundness = roundness(&polygon);
    println!("Roundness: {}", current_roundness);
}
