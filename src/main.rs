use std::iter::once;

use axum::Json;
use axum::{Router, routing::{get, post}};
use itertools::Itertools;
use nalgebra::{Isometry, Isometry2, Point2, Vector2};
use ncollide2d::bounding_volume::{BoundingSphere, BoundingVolume};
use ncollide2d::query::PointQuery;
use ncollide2d::shape::{Ball, ConvexPolygon, ConvexPolyhedron, Segment, Shape, ShapeHandle};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
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
    (incircle.radius() / circumcircle.radius()).powi(2)
}

#[derive(Serialize, Deserialize)]
struct Payload {
    points: Vec<(f64, f64)>,
}

#[tokio::main]
async fn main()
{
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/roundness", get(|| async { Json(Payload { points: vec![(0.0, 0.0), (1.0, 1.0)] }) }))
        .route("/roundness", post(api_roundness));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

#[derive(Serialize, Deserialize)]
struct RoundnessResult {
    roundness: f64
}

async fn api_roundness(Json(payload): Json<Payload>) -> Json<RoundnessResult> {
    let points = payload.points.iter().map(|(x, y)| Point2::new(*x, *y)).collect::<Vec<_>>();
    let polygon = ConvexPolygon::try_from_points(&points).unwrap();
    let roundness = roundness(&polygon);
    Json(RoundnessResult { roundness })
}
