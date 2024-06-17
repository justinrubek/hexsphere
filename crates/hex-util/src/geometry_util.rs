use crate::Hexagonish;
use glam::Vec3A;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
struct UnorderedTrio(u32, u32, u32);

fn trio(mut a: u32, mut b: u32, mut c: u32) -> UnorderedTrio {
    if a > b {
        std::mem::swap(&mut a, &mut b);
    }

    if b > c {
        std::mem::swap(&mut b, &mut c);

        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
    }

    UnorderedTrio(a, b, c)
}

pub struct GeometryData {
    pub points: Vec<Vec3A>,
    pub normals: Vec<Vec3A>,
    pub indices: Vec<u32>,
}

pub fn dual<'a>(
    ico_points: &[Vec3A],
    points_to_process: impl Iterator<Item = (u32, Option<&'a Hexagonish<u32>>)>,
    surrounding: &'a HashMap<u32, Hexagonish<u32>>,
    mut make_translation: impl FnMut(u32, u32, Hexagonish<u32>),
) -> GeometryData {
    let mut points = Vec::new();
    let mut indices = Vec::new();

    let mut triangle_set = HashMap::<UnorderedTrio, u32>::new();

    for (face, around) in points_to_process {
        let around = around.unwrap_or_else(|| &surrounding[&face]);
        let mid = points.len();

        let mut mid_val = Vec3A::ZERO;
        points.push(Vec3A::ZERO);
        for i in 0..around.len() {
            match triangle_set.entry(trio(face, around[i], around[(i + 1) % around.len()])) {
                Entry::Vacant(x) => {
                    let avg = ico_points[face as usize]
                        + ico_points[around[i] as usize]
                        + ico_points[around[(i + 1) % around.len()] as usize];
                    let avg = avg.normalize();
                    let idx = points.len();
                    points.push(avg);

                    x.insert(idx as _);

                    mid_val += avg;
                }
                Entry::Occupied(x) => mid_val += points[*x.get() as usize],
            }
        }

        mid_val /= around.len() as f32;

        points[mid] = mid_val;

        let mut edge_points = Hexagonish::new();

        for i in 0..around.len() {
            let a = mid as u32;
            let b = *triangle_set
                .get(&trio(face, around[i], around[(i + 1) % around.len()]))
                .unwrap();
            let c = *triangle_set
                .get(&trio(
                    face,
                    around[(i + 1) % around.len()],
                    around[(i + 2) % around.len()],
                ))
                .unwrap();

            edge_points.push(b);

            indices.extend_from_slice(&[a, b, c]);
        }

        make_translation(face, mid as u32, edge_points);
    }

    GeometryData {
        normals: points.iter().map(|x| x.normalize()).collect(),
        points,
        indices,
    }
}

pub fn steps_between(p1: Vec3A, p2: Vec3A, subdivisions: usize) -> usize {
    let q = p1.normalize().dot(p2.normalize()).acos() * (subdivisions + 1) as f32 * 2.0
        / (std::f32::consts::PI - 1.0);

    // There is some error in this calculation.
    // Through regression, the error is linear
    // and this un-applies it.

    (q - (0.032_834_187 * q - 0.483_033_93).round()).round() as usize
}

pub fn ray_unit_sphere_int(start: Vec3A, direction: Vec3A) -> Option<Vec3A> {
    let direction = direction.normalize();

    let lp = direction.dot(start);
    let pp = start.dot(start);

    let disc = lp * lp - pp + 1.0;

    if disc <= 0.0 {
        return None;
    }

    let t = -lp - disc.sqrt();

    Some(start + t * direction)
}
