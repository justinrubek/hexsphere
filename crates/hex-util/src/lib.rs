//! Example usage: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=6ee9f10224131656ea652f52718df8cf

use arrayvec::ArrayVec;
use bevy::{ecs::system::Resource, utils::Instant};
use glam::Vec3A;
use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, Index, IndexMut};

pub mod geometry_util;

use geometry_util::GeometryData;

#[cfg(feature = "algorithms")]
pub mod algorithms;

/// Either 5 or 6 elements.
pub type Hexagonish<T> = ArrayVec<T, 6>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chunked {
    subdivisions: usize,
}

impl Chunked {
    /// Yields the coordinates surrounding a given coordinate.
    pub fn surrounding(self, x: Coordinate) -> Hexagonish<Coordinate> {
        match x {
            Coordinate::Top => [
                coord(0, 0, 0),
                coord(1, 0, 0),
                coord(2, 0, 0),
                coord(3, 0, 0),
                coord(4, 0, 0),
            ]
            .as_ref()
            .try_into()
            .unwrap(),
            Coordinate::Bottom => [
                coord(4, self.subdivisions, 2 * self.subdivisions + 1),
                coord(3, self.subdivisions, 2 * self.subdivisions + 1),
                coord(2, self.subdivisions, 2 * self.subdivisions + 1),
                coord(1, self.subdivisions, 2 * self.subdivisions + 1),
                coord(0, self.subdivisions, 2 * self.subdivisions + 1),
            ]
            .as_ref()
            .try_into()
            .unwrap(),
            Coordinate::Inside { chunk, short, long } => {
                if self.subdivisions == 0 {
                    return if long == 0 {
                        [
                            Coordinate::Top,
                            coord((chunk + 4) % 5, 0, 0),
                            coord((chunk + 4) % 5, 0, 1),
                            coord(chunk, 0, 1),
                            coord((chunk + 1) % 5, 0, 0),
                        ]
                        .as_ref()
                        .try_into()
                        .unwrap()
                    } else {
                        [
                            coord(chunk, 0, 0),
                            coord((chunk + 4) % 5, 0, 1),
                            Coordinate::Bottom,
                            coord((chunk + 1) % 5, 0, 1),
                            coord((chunk + 1) % 5, 0, 0),
                        ]
                        .as_ref()
                        .try_into()
                        .unwrap()
                    };
                }

                if short == self.subdivisions {
                    return if long == 0 {
                        [
                            coord(chunk, short - 1, 0),
                            coord((chunk + 4) % 5, 0, short),
                            coord((chunk + 4) % 5, 0, short + 1),
                            coord(chunk, short, 1),
                            coord(chunk, short - 1, 1),
                        ]
                        .as_ref()
                        .try_into()
                        .unwrap()
                    } else if long == self.subdivisions + 1 {
                        [
                            coord(chunk, short, long - 1),
                            coord((chunk + 4) % 5, 0, short + long),
                            coord(chunk, short, long + 1),
                            coord(chunk, short - 1, long + 1),
                            coord(chunk, short - 1, long),
                        ]
                        .as_ref()
                        .try_into()
                        .unwrap()
                    } else if long <= self.subdivisions {
                        [
                            coord(chunk, short, long - 1),
                            coord((chunk + 4) % 5, 0, short + long),
                            coord((chunk + 4) % 5, 0, short + long + 1),
                            coord(chunk, short, long + 1),
                            coord(chunk, short - 1, long + 1),
                            coord(chunk, short - 1, long),
                        ]
                        .into()
                    } else if long <= 2 * self.subdivisions {
                        [
                            coord(chunk, short, long - 1),
                            coord(
                                (chunk + 4) % 5,
                                long - self.subdivisions - 2,
                                self.subdivisions * 2 + 1,
                            ),
                            coord(
                                (chunk + 4) % 5,
                                long - self.subdivisions - 1,
                                self.subdivisions * 2 + 1,
                            ),
                            coord(chunk, short, long + 1),
                            coord(chunk, short - 1, long + 1),
                            coord(chunk, short - 1, long),
                        ]
                        .into()
                    } else {
                        [
                            coord(chunk, short, long - 1),
                            coord((chunk + 4) % 5, short - 1, long),
                            coord((chunk + 4) % 5, short, long),
                            Coordinate::Bottom,
                            coord((chunk + 1) % 5, short, long),
                            coord(chunk, short - 1, long),
                        ]
                        .into()
                    };
                }

                if short == 0 {
                    return if long == 0 {
                        [
                            Coordinate::Top,
                            coord((chunk + 4) % 5, 0, 0),
                            coord((chunk + 4) % 5, 0, 1),
                            coord(chunk, 1, 0),
                            coord(chunk, 0, 1),
                            coord((chunk + 1) % 5, 0, 0),
                        ]
                        .into()
                    } else if long <= self.subdivisions {
                        [
                            coord(chunk, 0, long - 1),
                            coord(chunk, 1, long - 1),
                            coord(chunk, 1, long),
                            coord(chunk, 0, long + 1),
                            coord((chunk + 1) % 5, long, 0),
                            coord((chunk + 1) % 5, long - 1, 0),
                        ]
                        .into()
                    } else if long <= 2 * self.subdivisions {
                        [
                            coord(chunk, 0, long - 1),
                            coord(chunk, 1, long - 1),
                            coord(chunk, 1, long),
                            coord(chunk, 0, long + 1),
                            coord((chunk + 1) % 5, self.subdivisions, long - self.subdivisions),
                            coord(
                                (chunk + 1) % 5,
                                self.subdivisions,
                                (long - self.subdivisions) - 1,
                            ),
                        ]
                        .into()
                    } else {
                        [
                            coord(chunk, 0, long - 1),
                            coord(chunk, 1, long - 1),
                            coord(chunk, 1, long),
                            coord((chunk + 1) % 5, self.subdivisions, self.subdivisions + 2),
                            coord((chunk + 1) % 5, self.subdivisions, self.subdivisions + 1),
                            coord((chunk + 1) % 5, self.subdivisions, self.subdivisions),
                        ]
                        .as_ref()
                        .try_into()
                        .unwrap()
                    };
                }

                if long == 0 {
                    [
                        coord(chunk, short - 1, 0),
                        coord((chunk + 4) % 5, 0, short),
                        coord((chunk + 4) % 5, 0, short + 1),
                        coord(chunk, short + 1, 0),
                        coord(chunk, short, 1),
                        coord(chunk, short - 1, 1),
                    ]
                    .into()
                } else if long <= 2 * self.subdivisions {
                    [
                        coord(chunk, short, long - 1),
                        coord(chunk, short + 1, long - 1),
                        coord(chunk, short + 1, long),
                        coord(chunk, short, long + 1),
                        coord(chunk, short - 1, long + 1),
                        coord(chunk, short - 1, long),
                    ]
                    .into()
                } else {
                    [
                        coord(chunk, short, long - 1),
                        coord(chunk, short + 1, long - 1),
                        coord(chunk, short + 1, long),
                        coord(
                            (chunk + 1) % 5,
                            self.subdivisions,
                            self.subdivisions + 2 + short,
                        ),
                        coord(
                            (chunk + 1) % 5,
                            self.subdivisions,
                            self.subdivisions + 1 + short,
                        ),
                        coord(chunk, short - 1, long),
                    ]
                    .into()
                }
            }
        }
    }

    pub fn is_valid(self, coord: Coordinate) -> bool {
        match coord {
            Coordinate::Top | Coordinate::Bottom => true,
            Coordinate::Inside { chunk, short, long } => {
                chunk < 5 && short <= self.subdivisions && long <= 2 * self.subdivisions + 1
            }
        }
    }

    pub fn subdivisions(self) -> usize {
        self.subdivisions
    }

    /// Parameters:
    /// - `from` and `to` must be adjacent.
    /// - `choose` takes in:
    ///    1) A coordinate for a hexagon
    ///    2) A coordinate for an adjacent pentagon
    ///    3) Two possible directions to go in
    ///
    ///    And returns which of the two directions to go.
    pub fn continue_line<F>(self, from: Coordinate, to: Coordinate, choose: F) -> LineCont<F>
    where
        F: FnMut(Coordinate, Coordinate, (Coordinate, Coordinate)) -> Coordinate,
    {
        LineCont {
            sphere: self,
            prev: from,
            next: to,
            choose,
        }
    }

    pub fn find_blobs(self, coords: impl Iterator<Item = Coordinate>) -> Option<Vec<Blob>> {
        let mut to_explore = vec![];
        let mut all = coords.collect::<HashSet<_>>();

        let mut yielded = Vec::new();

        while !all.is_empty() {
            let first = *all.iter().next().unwrap();

            to_explore.push(first);

            let mut current_contents = HashSet::new();
            current_contents.insert(first);

            let mut bordered = Vec::new();

            while let Some(next) = to_explore.pop() {
                let mut bordered_by_nothing = false;

                for around in self.surrounding(next) {
                    let exists = all.contains(&around);

                    if exists && !current_contents.contains(&around) {
                        to_explore.push(around);
                        current_contents.insert(around);
                    }

                    bordered_by_nothing |= !exists;
                }

                if bordered_by_nothing {
                    bordered.push(next);
                }
            }

            all.retain(|x| !current_contents.contains(x));

            yielded.push(Blob {
                contents: current_contents,
                borders: bordered,
            });
        }

        Some(yielded)
    }

    pub fn blob_borders(self, blob: &Blob) -> Option<Vec<Vec<Coordinate>>> {
        self.find_blobs(blob.borders.iter().copied())
            .map(|x| x.into_iter().map(|blob| blob.borders).collect::<Vec<_>>())
    }

    pub fn ring_order(self, coordinates: &mut [Coordinate], inside: &HashSet<Coordinate>) {
        let mut visited = HashSet::new();
        let all = coordinates.iter().copied().collect::<HashSet<_>>();
        let mut ordered = Vec::new();

        let mut current = coordinates[0];

        visited.insert(current);

        ordered.push(current);

        while visited.len() != all.len() {
            let next = self
                .surrounding(current)
                .into_iter()
                .filter(|x| all.contains(x))
                .find(|x| !visited.contains(x))
                .unwrap();

            visited.insert(next);
            ordered.push(next);
            current = next;
        }

        coordinates.copy_from_slice(&ordered);

        // Test for correct chirality.
        let first = coordinates[0];
        let second = coordinates[1];

        let first_surrounding = self.surrounding(first);
        let common_out = self
            .surrounding(second)
            .into_iter()
            .find(|x| first_surrounding.contains(x) && !inside.contains(x))
            .unwrap();

        // Since it's outside, the first two should have reversed winding for it.
        let out_surrounding = self.surrounding(common_out);

        let idx = out_surrounding.iter().position(|x| *x == second).unwrap();
        if first != out_surrounding[(idx + 1) % out_surrounding.len()] {
            coordinates.reverse();
        }
    }

    pub fn iter_all(self) -> impl Iterator<Item = Coordinate> {
        [Coordinate::Top, Coordinate::Bottom]
            .into_iter()
            .chain((0..5u8).flat_map(move |chunk| {
                (0..self.subdivisions + 1).flat_map(move |short| {
                    (0..(self.subdivisions + 1) * 2).map(move |long| Coordinate::Inside {
                        chunk,
                        short,
                        long,
                    })
                })
            }))
    }
}

/// Organizes data on a hexagon tiled sphere.
///
/// This does not deal with geometry at all,
/// however the algorithms in its impl allow you
/// to tie that into its creation.
#[derive(Clone, Debug, PartialEq, Resource)]
pub struct Hexasphere<T> {
    inner: Chunked,
    top: T,
    bottom: T,
    chunks: [Vec<T>; 5],
}

impl<T> Deref for Hexasphere<T> {
    type Target = Chunked;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Hexasphere<T> {
    /// Assumes that the geometry is from the hexasphere
    /// geometry crate to organize itself.
    ///
    /// # Arguments
    /// - `subdivisions` is the number of subdivisions applied
    ///   to generate the geometry.
    /// - `indices` is the indices yielded by `hexasphere`.
    /// - `make` allows the user to map an index into the
    ///   hexasphere geometry (an old index) into a new coordinate,
    ///   as well as provide an entry for that coordinate.
    ///
    /// # Returns
    /// - The organizational structure, `Self`.
    /// - The adjacency map.
    pub fn from_hexasphere_geometry(
        subdivisions: usize,
        indices: &[u32],
        make: impl FnMut(u32, Coordinate) -> T,
    ) -> (Self, HashMap<u32, Hexagonish<u32>>) {
        let mut coordinate_store = HashMap::new();
        make_coordinate_store(indices, &mut coordinate_store);

        (
            Self::make_from_surrounding(subdivisions, &coordinate_store, make),
            coordinate_store,
        )
    }

    /// Creates the organization structure, `Hexasphere`.
    ///
    /// # Arguments
    /// - `subdivisions` is the number of subdivisions the sphere was made with.
    /// - `coordinate_store` is the adjacency map for the coordinates.
    ///   Generate this with [`make_coordinate_store`]. Assumes that for a
    ///   given index, the yielded list preserves the winding order.
    /// - `make` allows the user to translate from an old index into
    ///   a `Coordinate`, as well as providing an entry for that coordinate.
    pub fn make_from_surrounding(
        subdivisions: usize,
        coordinate_store: &HashMap<u32, Hexagonish<u32>>,
        mut make: impl FnMut(u32, Coordinate) -> T,
    ) -> Self {
        let top = make(0, Coordinate::Top);
        let bottom = make(11, Coordinate::Bottom);

        let mut chunks = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];

        let rotate_by = |previous: u32, this: u32, by: isize| {
            let list = coordinate_store.get(&this).unwrap();
            let idx = list.iter().position(|&x| x == previous).unwrap();
            list[(idx as isize + by).rem_euclid(list.len() as isize) as usize]
        };

        for (chunk, data) in chunks.iter_mut().enumerate() {
            let mut short_prev = 0;
            let mut short_root = coordinate_store.get(&0).unwrap()[chunk];

            for short in 0..subdivisions + 1 {
                data.push(make(short_root, coord(chunk as u8, short, 0)));

                let mut long_root = rotate_by(short_prev, short_root, -2);
                let mut long_prev = short_root;

                for long in 1..2 * (subdivisions + 1) {
                    data.push(make(long_root, coord(chunk as u8, short, long)));

                    let new_long_root = rotate_by(long_prev, long_root, -3);
                    long_prev = long_root;
                    long_root = new_long_root;
                }

                let new_short_root = rotate_by(short_prev, short_root, -3);
                short_prev = short_root;
                short_root = new_short_root;
            }
        }

        Self {
            inner: Chunked { subdivisions },
            top,
            bottom,
            chunks,
        }
    }

    /// Generates duals of geometry given indices and points.
    ///
    /// # Arguments
    /// - `subdivisions`: Number of subdivisions the data was created with.
    /// - `indices`: Indices of IcoSphere.
    /// - `ico_points`: Points of IcoSphere.
    /// - `make_temporary`: Allows user data to be created.
    /// - `make`: Takes 5 parameters:
    ///     - The index of the center of the new face in the new geometry.
    ///     - The indices of the vertices of the new polygon.
    ///     - The coordinate that this polygon is assigned.
    ///     - The geometry data, should you want to modify or read it.
    ///     - The temporary data generated by `make_temporary`.
    ///
    ///   Returns: the data to go in the slot of that coordinate.
    ///
    /// # Returns:
    /// - The organization structure `Hexasphere`.
    /// - The new `GeometryData`.
    /// - The temporary data.
    pub fn make_and_dual<E>(
        subdivisions: usize,
        indices: &[u32],
        ico_points: &[Vec3A],
        make_temporary: impl FnOnce(&GeometryData) -> E,
        mut make: impl FnMut(u32, Hexagonish<u32>, Coordinate, &mut GeometryData, &mut E) -> T,
    ) -> (Self, GeometryData, E) {
        let start = Instant::now();
        let mut coordinate_store = HashMap::new();
        make_coordinate_store(indices, &mut coordinate_store);
        println!("[{}:{}] CStore: {:?}", file!(), line!(), start.elapsed());

        let start = Instant::now();
        let mut convert_to_dual_space = HashMap::new();

        let mut dual_data = geometry_util::dual(
            ico_points,
            coordinate_store
                .iter()
                .map(|(&x, around)| (x, Some(around))),
            &coordinate_store,
            |old, new, edges| {
                convert_to_dual_space.insert(old, (new, edges));
            },
        );
        println!("[{}:{}] Dual: {:?}", file!(), line!(), start.elapsed());

        let start = Instant::now();
        let mut temp = make_temporary(&dual_data);

        let surrounding =
            Self::make_from_surrounding(subdivisions, &coordinate_store, |old, coord| {
                let (new, edges) = convert_to_dual_space.get(&old).unwrap().clone();

                make(new, edges, coord, &mut dual_data, &mut temp)
            });
        println!("[{}:{}] Make: {:?}", file!(), line!(), start.elapsed());

        (surrounding, dual_data, temp)
    }

    /// This duals the geometry, generates the organization structure, and also creates chunks.
    ///
    /// # Arguments
    /// - `subdivisions`: Number of subdivisions.
    /// - `next_indices`: Asks for the next set of indices (this is usually a call to
    ///   `hexasphere`'s get triangle indices function.
    /// - `ico_points` is the points of the sphere.
    /// - `make_temporary` generates temporary data which will be processed alongside points.
    /// - `make` takes as its first argument the list of buffers to which this
    ///   coordinate belongs, (the `usize`), the index into each of the buffers (the `u32`),
    ///   and the indices of the vertices of the polygon in each buffer, (the `Hexagonish<u32>`).
    ///   Afterwards, it takes the coordinate to be processed, the new geometry data, and the
    ///   temporary data.
    pub fn chunked_dual<E>(
        subdivisions: usize,
        mut next_indices: impl FnMut(&mut Vec<u32>),
        ico_points: &[Vec3A],
        make_temporary: impl FnOnce(&[GeometryData]) -> E,
        mut make: impl FnMut(
            Hexagonish<(usize, u32, Hexagonish<u32>)>,
            Coordinate,
            &[GeometryData],
            &mut E,
        ) -> T,
    ) -> (Self, Vec<GeometryData>, E) {
        let mut acc_coordinate_store = HashMap::new();

        let mut convert_to_dual_space =
            HashMap::<u32, Hexagonish<(usize, u32, Hexagonish<u32>)>>::new();

        let mut resulting_chunks = Vec::new();

        let mut indices = Vec::new();

        next_indices(&mut indices);

        let mut sets = Vec::with_capacity(20);

        while !indices.is_empty() {
            sets.push(indices.iter().copied().collect::<HashSet<_>>());
            make_coordinate_store(&indices, &mut acc_coordinate_store);
            indices.clear();
            next_indices(&mut indices);
        }

        for set in sets {
            let dual_data = geometry_util::dual(
                ico_points,
                set.into_iter().map(|x| (x, None)),
                &acc_coordinate_store,
                |old, new, edges| {
                    convert_to_dual_space.entry(old).or_default().push((
                        resulting_chunks.len(),
                        new,
                        edges,
                    ));
                },
            );

            resulting_chunks.push(dual_data);
        }

        let mut temp = make_temporary(&resulting_chunks);

        let surrounding =
            Self::make_from_surrounding(subdivisions, &acc_coordinate_store, |old, coord| {
                let results = convert_to_dual_space.remove(&old).unwrap();

                make(results, coord, &resulting_chunks, &mut temp)
            });

        (surrounding, resulting_chunks, temp)
    }

    pub fn chunked(&self) -> Chunked {
        self.inner
    }

    pub fn iter(&'_ self, coord: Coordinate) -> impl Iterator<Item = (&'_ T, Coordinate)> {
        self.surrounding(coord).into_iter().map(|x| (&self[x], x))
    }

    pub fn change_type<Q>(&self, mut to: impl FnMut(&T) -> Q) -> Hexasphere<Q> {
        Hexasphere {
            inner: self.inner,
            top: to(&self.top),
            bottom: to(&self.bottom),
            chunks: [0, 1, 2, 3, 4].map(|x| self.chunks[x].iter().map(&mut to).collect::<Vec<_>>()),
        }
    }

    pub fn all(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.top)
            .chain(std::iter::once(&self.bottom))
            .chain(self.chunks.iter().flatten())
    }

    pub fn all_mut(&mut self) -> impl Iterator<Item = &mut T> {
        std::iter::once(&mut self.top)
            .chain(std::iter::once(&mut self.bottom))
            .chain(self.chunks.iter_mut().flat_map(|x| x.iter_mut()))
    }

    pub fn get_many<const N: usize>(&self, coordinates: [Coordinate; N]) -> [&T; N] {
        coordinates.map(|x| &self[x])
    }

    pub fn get_many_mut<const N: usize>(
        &mut self,
        coordinates: [Coordinate; N],
    ) -> Option<[&mut T; N]> {
        for val in coordinates {
            assert!(self.inner.is_valid(val));
            for test in coordinates {
                if val == test {
                    return None;
                }
            }
        }

        let len = self.chunks[0].len();

        let top = (&mut self.top) as *mut T;
        let bottom = (&mut self.bottom) as *mut T;
        let [c0, c1, c2, c3, c4] = &mut self.chunks;
        let coords = [c0, c1, c2, c3, c4].map(|x| (**x).as_mut_ptr());

        let vals = coordinates.map(|x| {
            // SAFETY: No two distinct coordinates refer to the same value
            // SAFETY: No two coordinates in the coordinates we're accessing are the same.
            unsafe {
                match x {
                    Coordinate::Top => &mut *top,
                    Coordinate::Bottom => &mut *bottom,
                    Coordinate::Inside { chunk, short, long } => {
                        let idx = short * 2 * (self.subdivisions + 1) + long;
                        if idx > len {
                            panic!("Coordinate is invalid!");
                        }
                        let ptr = coords[chunk as usize];
                        &mut *ptr.add(idx)
                    }
                }
            }
        });

        Some(vals)
    }
}

pub struct LineCont<F>
where
    F: FnMut(Coordinate, Coordinate, (Coordinate, Coordinate)) -> Coordinate,
{
    sphere: Chunked,
    prev: Coordinate,
    next: Coordinate,
    choose: F,
}

impl<F> Iterator for LineCont<F>
where
    F: FnMut(Coordinate, Coordinate, (Coordinate, Coordinate)) -> Coordinate,
{
    type Item = Coordinate;

    fn next(&mut self) -> Option<Self::Item> {
        let surrounding = self.sphere.surrounding(self.next);
        let choice_a = rotate(self.prev, 3, &surrounding);
        let choice_b = rotate(self.prev, -3, &surrounding);

        let choice = if choice_a != choice_b {
            (self.choose)(self.prev, self.next, (choice_a, choice_b))
        } else {
            choice_a
        };

        self.prev = self.next;
        self.next = choice;

        Some(choice)
    }
}

fn rotate(previous: Coordinate, by: isize, surrounding: &[Coordinate]) -> Coordinate {
    let idx = surrounding.iter().position(|&x| x == previous).unwrap();
    surrounding[(idx as isize + by).rem_euclid(surrounding.len() as isize) as usize]
}

impl<T> Index<Coordinate> for Hexasphere<T> {
    type Output = T;

    fn index(&self, index: Coordinate) -> &Self::Output {
        match index {
            Coordinate::Top => &self.top,
            Coordinate::Bottom => &self.bottom,
            Coordinate::Inside { chunk, short, long } => {
                &self.chunks[chunk as usize][short * 2 * (self.subdivisions + 1) + long]
            }
        }
    }
}

impl<T> IndexMut<Coordinate> for Hexasphere<T> {
    fn index_mut(&mut self, index: Coordinate) -> &mut Self::Output {
        match index {
            Coordinate::Top => &mut self.top,
            Coordinate::Bottom => &mut self.bottom,
            Coordinate::Inside { chunk, short, long } => {
                let subdivisions = self.subdivisions;
                &mut self.chunks[chunk as usize][short * 2 * (subdivisions + 1) + long]
            }
        }
    }
}

impl<'a, T> Index<&'a Coordinate> for Hexasphere<T> {
    type Output = T;

    fn index(&self, index: &'a Coordinate) -> &Self::Output {
        match *index {
            Coordinate::Top => &self.top,
            Coordinate::Bottom => &self.bottom,
            Coordinate::Inside { chunk, short, long } => {
                &self.chunks[chunk as usize][short * 2 * (self.subdivisions + 1) + long]
            }
        }
    }
}

impl<'a, T> IndexMut<&'a Coordinate> for Hexasphere<T> {
    fn index_mut(&mut self, index: &'a Coordinate) -> &mut Self::Output {
        match *index {
            Coordinate::Top => &mut self.top,
            Coordinate::Bottom => &mut self.bottom,
            Coordinate::Inside { chunk, short, long } => {
                let subdivisions = self.subdivisions;
                &mut self.chunks[chunk as usize][short * 2 * (subdivisions + 1) + long]
            }
        }
    }
}

/// Coordinate on a hexasphere.
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash)]
pub enum Coordinate {
    #[default]
    Top,
    Bottom,
    Inside {
        /// In `0..=4`.
        chunk: u8,
        /// In `0..subdiv + 1`.
        short: usize,
        /// In `0..(subdiv + 1) * 2`.
        long: usize,
    },
}

impl Debug for Coordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Coordinate::Top => write!(f, "Top"),
            Coordinate::Bottom => write!(f, "Bottom"),
            Coordinate::Inside { chunk, short, long } => {
                write!(f, "I({}, {}, {})", chunk, short, long)
            }
        }
    }
}

impl Ord for Coordinate {
    fn cmp(&self, other: &Self) -> Ordering {
        use Coordinate::*;

        match (self, other) {
            (Top, _) => Ordering::Greater,
            (_, Top) => Ordering::Less,
            (Bottom, Bottom) => Ordering::Equal,
            (Bottom, Inside { .. }) => Ordering::Greater,
            (Inside { .. }, Bottom) => Ordering::Less,
            (
                Inside {
                    chunk: c1,
                    short: s1,
                    long: l1,
                },
                Inside {
                    chunk: c2,
                    short: s2,
                    long: l2,
                },
            ) => match c1.cmp(c2) {
                Ordering::Equal => match s1.cmp(s2) {
                    Ordering::Equal => l1.cmp(l2),
                    x => x,
                },
                x => x,
            },
        }
    }
}

impl PartialOrd for Coordinate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Short form to create a `Coordinate`.
pub fn coord(chunk: u8, short: usize, long: usize) -> Coordinate {
    Coordinate::Inside { chunk, short, long }
}

/// Creates an adjacency map in place from this set of coordinates.
///
/// Preserves order and eliminates duplicates in each list.
pub fn make_coordinate_store(
    indices: &[u32],
    coordinate_store: &mut HashMap<u32, Hexagonish<u32>>,
) {
    assert_eq!(indices.len() % 3, 0);
    indices.chunks(3).for_each(|x| {
        if let &[a, b, c] = x {
            let mut store_entry = |i, j, k| match coordinate_store.entry(i) {
                Entry::Vacant(x) => {
                    x.insert(ArrayVec::from_iter([j, k]));
                }
                Entry::Occupied(x) => {
                    let list = x.into_mut();
                    if let Some(idx_j) = list.iter().position(|&z| z == j) {
                        if list[(idx_j + 1) % list.len()] != k {
                            list.insert(idx_j + 1, k);
                        }
                    } else if let Some(idx_k) = list.iter().position(|&z| z == k) {
                        list.insert(idx_k, j);
                    } else {
                        list.push(j);
                        list.push(k);
                    }
                }
            };

            // Order of the entries in the arrays matters!
            store_entry(a, b, c);
            store_entry(b, c, a);
            store_entry(c, a, b);
        }
    });
}

#[derive(Debug)]
pub struct Blob {
    pub contents: HashSet<Coordinate>,
    pub borders: Vec<Coordinate>,
}

#[cfg(test)]
mod tests {
    use crate::{Chunked, Coordinate};
    use std::collections::{HashMap, HashSet};

    #[test]
    fn coord_surrounding() {
        let hsphere = Chunked { subdivisions: 3 };

        let mut to_visit = vec![Coordinate::Top];
        let mut visited = HashMap::<Coordinate, HashSet<Coordinate>>::new();

        while let Some(x) = to_visit.pop() {
            let surrounding = hsphere.surrounding(x);
            let set = surrounding.iter().copied().collect::<HashSet<_>>();

            visited.insert(x, set);
            surrounding.iter().copied().for_each(|z| {
                if !hsphere.is_valid(z) {
                    panic!(
                        "Coordinate {:?} yielded {:?} with bad {:?}.",
                        x, surrounding, z
                    );
                }
                if !visited.contains_key(&z) {
                    to_visit.push(z);
                }
            })
        }

        for (x, set) in &visited {
            assert!(!set.contains(x));

            set.iter().for_each(|z| {
                assert!(visited.get(z).unwrap().contains(x));
                assert_ne!(z, x);
            });
        }
    }
}
