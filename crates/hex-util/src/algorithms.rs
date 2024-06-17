use crate::{Coordinate, Hexagonish, Hexasphere};
use glam::Vec3A;
use pathfinding::prelude::astar as ext_astar;

///
/// Runs the `A*` algorithm from the `pathfinding` crate.
///
/// - `point_a` is the beginning.
/// - `point_b` is the destination.
/// - `sphere` is the grid system upon which to run it.
/// - `weight(sphere, X, Y)` is the cost of moving from tile X to tile Y, or
///     `None` if the movement is impossible.
/// - `distance(sphere, X, Y)` is the distance from tile X to tile Y. This is
///     usually just the distance along the surface of the sphere.
///
pub fn astar<T>(
    point_a: Coordinate,
    point_b: Coordinate,
    sphere: &Hexasphere<T>,
    mut weight: impl FnMut(&Hexasphere<T>, Coordinate, Coordinate) -> Option<usize>,
    mut distance: impl FnMut(&Hexasphere<T>, Coordinate, Coordinate) -> usize,
) -> Option<(Vec<Coordinate>, usize)> {
    ext_astar(
        &point_a,
        |&x| {
            let surrounding = sphere.surrounding(x);
            surrounding
                .into_iter()
                .filter_map(|next| Some((next, weight(sphere, x, next)?)))
                .collect::<Hexagonish<_>>()
        },
        |&x| distance(sphere, x, point_b),
        |&result| result == point_b,
    )
}

pub fn find_point<T>(
    point_on_sphere: Vec3A,
    sphere: &Hexasphere<T>,
    position: impl Fn(&Hexasphere<T>, Coordinate) -> Vec3A,
) -> Coordinate {
    let distance = |&x: &Coordinate| 1.0 - point_on_sphere.dot(position(sphere, x));

    let mut current = Coordinate::Top;
    let mut best_distance = distance(&current);

    'outer: loop {
        let surrounding = sphere.surrounding(current);

        for i in surrounding {
            let dist = distance(&i);
            if dist < best_distance {
                best_distance = dist;
                current = i;
                continue 'outer;
            }
        }

        return current;
    }
}
