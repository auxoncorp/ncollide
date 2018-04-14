use std::mem;
use na::{self, Real};
use math::{Point, Isometry};
use query::{PointQuery, PointQueryWithLocation};
use query::algorithms::simplex::Simplex;
use shape::{Segment, SegmentPointLocation, Triangle, TrianglePointLocation};

/// A simplex of dimension up to 2 using Voronoï regions for computing point projections.
pub struct VoronoiSimplex2<N: Real, T: 'static + Copy + Send + Sync> {
    prev_vertices: [usize; 3],
    prev_dim: usize,
    prev_proj: [N; 2],

    vertices: [Point<N>; 3],
    data: [T; 3],
    proj: [N; 2],
    dim: usize,
}

impl<N: Real, T: 'static + Copy + Send + Sync> VoronoiSimplex2<N, T> {
    /// Crates a new empty simplex.
    pub fn new() -> VoronoiSimplex2<N, T> {
        VoronoiSimplex2 {
            prev_vertices: [0, 1, 2],
            prev_proj: [N::zero(); 2],
            prev_dim: 0,
            vertices: [Point::origin(); 3],
            data: [unsafe { mem::uninitialized() }; 3],
            proj: [N::zero(); 2],
            dim: 0,
        }
    }

    fn swap(&mut self, i1: usize, i2: usize) {
        self.vertices.swap(i1, i2);
        self.data.swap(i1, i2);
        self.prev_vertices.swap(i1, i2);
    }
}

/// Trait of a simplex usable by the GJK algorithm.
impl<N: Real, T: 'static + Copy + Send + Sync> Simplex<N, T> for VoronoiSimplex2<N, T> {
    fn reset(&mut self, pt: Point<N>, data: T) {
        self.prev_dim = 0;
        self.dim = 0;
        self.vertices[0] = pt;
        self.data[0] = data;
    }

    fn add_point(&mut self, pt: Point<N>, data: T) -> bool {
        self.prev_dim = self.dim;
        self.prev_proj = self.proj;
        self.prev_vertices = [0, 1, 2];

        for i in 0..self.dim + 1 {
            if self.vertices[i].coords == pt.coords {
                return false;
            }
        }

        self.dim += 1;
        self.vertices[self.dim] = pt;
        self.data[self.dim] = data;
        return true;
    }

    fn proj_coord(&self, i: usize) -> N {
        assert!(i <= self.dim, "Index out of bounds.");
        self.proj[i]
    }

    fn point(&self, i: usize) -> &Point<N> {
        assert!(i <= self.dim, "Index out of bounds.");
        &self.vertices[i]
    }

    fn data(&self, i: usize) -> &T {
        assert!(i <= self.dim, "Index out of bounds.");
        &self.data[i]
    }
    
    fn prev_proj_coord(&self, i: usize) -> N {
        assert!(i <= self.dim, "Index out of bounds.");
        self.prev_proj[i]
    }
    
    fn prev_point(&self, i: usize) -> &Point<N> {
        assert!(i <= self.prev_dim, "Index out of bounds.");
        &self.vertices[self.prev_vertices[i]]
    }

    fn prev_data(&self, i: usize) -> &T {
        assert!(i <= self.prev_dim, "Index out of bounds.");
        &self.data[self.prev_vertices[i]]
    }

    fn project_origin_and_reduce(&mut self) -> Point<N> {
        if self.dim == 0 {
            self.proj[0] = N::one();
            self.vertices[0]
        } else if self.dim == 1 {
            // FIXME: NLL
            let (proj, location) = {
                let seg = Segment::from_array3(&self.vertices);
                seg.project_point_with_location(&Isometry::identity(), &Point::origin(), true)
            };

            match location {
                SegmentPointLocation::OnVertex(0) => {
                    self.proj[0] = N::one();
                    self.dim = 0;
                }
                SegmentPointLocation::OnVertex(1) => {
                    self.proj[0] = N::one();
                    self.swap(0, 1);
                    self.dim = 0;
                }
                SegmentPointLocation::OnEdge(coords) => {
                    self.proj = coords;
                },
                _ => unreachable!()
            }

            proj.point
        } else {
            assert!(self.dim == 2);
            // FIXME: NLL
            let (proj, location) = {
                let tri = Triangle::from_array(&self.vertices);
                tri.project_point_with_location(&Isometry::identity(), &Point::origin(), true)
            };

            match location {
                TrianglePointLocation::OnVertex(i) => {
                    self.swap(0, i);
                    self.proj[0] = N::one();
                    self.dim = 0;
                }
                TrianglePointLocation::OnEdge(0, coords) => {
                    self.proj = coords;
                    self.dim = 1;
                }
                TrianglePointLocation::OnEdge(1, coords) => {
                    self.swap(0, 2);
                    self.proj[0] = coords[1];
                    self.proj[1] = coords[0];
                    self.dim = 1;
                }
                TrianglePointLocation::OnEdge(2, coords) => {
                    self.swap(1, 2);
                    self.proj = coords;
                    self.dim = 1;
                }
                _ => {}
            }

            proj.point
        }
    }

    fn project_origin(&mut self) -> Point<N> {
        if self.dim == 0 {
            self.vertices[0]
        } else if self.dim == 1 {
            let seg = Segment::from_array3(&self.vertices);
            seg.project_point(&Isometry::identity(), &Point::origin(), true).point
        } else {
            assert!(self.dim == 2);
            let tri = Triangle::from_array(&self.vertices);
            tri.project_point(&Isometry::identity(), &Point::origin(), true).point
        }
    }

    fn contains_point(&self, pt: &Point<N>) -> bool {
        for i in 0..self.dim + 1 {
            if self.vertices[i] == *pt {
                return true;
            }
        }

        false
    }

    fn dimension(&self) -> usize {
        self.dim
    }

    fn prev_dimension(&self) -> usize {
        self.prev_dim
    }

    fn max_sq_len(&self) -> N {
        let mut max_sq_len = na::zero();

        for i in 0..self.dim + 1 {
            let norm = na::norm_squared(&self.vertices[i].coords);

            if norm > max_sq_len {
                max_sq_len = norm
            }
        }

        max_sq_len
    }

    fn modify_pnts(&mut self, f: &Fn(&mut Point<N>)) {
        for i in 0..self.dim + 1 {
            f(&mut self.vertices[i])
        }
    }
}
