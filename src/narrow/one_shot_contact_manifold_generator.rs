use std::num::One;
use nalgebra::traits::basis::Basis;
use nalgebra::traits::cross::Cross;
use nalgebra::traits::rotation;
use nalgebra::traits::vector::{AlgebraicVecExt, Vec};
use nalgebra::traits::rotation::Rotation;
use nalgebra::traits::transformation::Transform;
use nalgebra::traits::translation::Translation;
use narrow::collision_detector::CollisionDetector;
use narrow::incremental_contact_manifold_generator::IncrementalContactManifoldGenerator;
use contact::Contact;

/// This is an hybrid contact manifold genarator. Whenever a new contact is detected (i.e. when the
/// current manifold is empty) a full manifold is generated. Then, the manifold is incrementally
/// updated by the `IncrementalContactManifoldGenerator`.
pub struct OneShotContactManifoldGenerator<CD, N, LV, AV, M> {
    priv sub_detector: IncrementalContactManifoldGenerator<CD, N, LV>
}

impl<CD, N, LV, AV, M> OneShotContactManifoldGenerator<CD, N, LV, AV, M> {
    /// Creates a new one shot contact manifold generator.
    pub fn new(prediction: N, cd: CD) -> OneShotContactManifoldGenerator<CD, N, LV, AV, M> {
        OneShotContactManifoldGenerator {
            sub_detector: IncrementalContactManifoldGenerator::new(prediction, cd)
        }
    }
}

impl<CD: CollisionDetector<N, LV, M, G1, G2>,
     G1,
     G2,
     N:  Clone + Num + Ord + NumCast + Algebraic,
     LV: Clone + AlgebraicVecExt<N> + Cross<AV> + ApproxEq<N> + ToStr,
     AV: Vec<N> + ToStr,
     M:  Rotation<AV> + Transform<LV> + Translation<LV> + One>
CollisionDetector<N, LV, M, G1, G2> for OneShotContactManifoldGenerator<CD, N, LV, AV, M> {
    fn update(&mut self, m1: &M, g1: &G1, m2: &M, g2: &G2) {
        if self.sub_detector.num_coll() == 0 {
            // do the one-shot manifold generation
            match self.sub_detector.get_sub_collision(m1, g1, m2, g2) {
                Some(coll) => {
                    do coll.normal.orthonormal_subspace_basis |b| {
                        let mut rot_axis: AV = coll.normal.cross(&b);

                        // first perturbation
                        rot_axis = rot_axis * NumCast::from(0.01);

                        let rot_mat: M = rotation::rotated_wrt_center(m1, &rot_axis);

                        self.sub_detector.add_new_contacts(&rot_mat, g1, m2, g2);

                        // second perturbation (opposite direction)
                        let rot_mat = rotation::rotated_wrt_center(m1, &-rot_axis);

                        self.sub_detector.add_new_contacts(&rot_mat, g1, m2, g2);

                        true
                    }

                    self.sub_detector.update_contacts(m1, m2);
                },
                None => { } // no collision
            }
        }
        else {
            // otherwise, let the incremental manifold do its job
            self.sub_detector.update(m1, g1, m2, g2)
        }
    }

    #[inline]
    fn num_coll(&self) -> uint {
        self.sub_detector.num_coll()
    }

    #[inline]
    fn colls(&self, out_colls: &mut ~[Contact<N, LV>]) {
        self.sub_detector.colls(out_colls)
    }

    #[inline]
    fn toi(m1: &M, dir: &LV, g1: &G1, m2: &M, g2: &G2) -> Option<N> {
        CollisionDetector::toi::<N, LV, M, G1, G2, CD>(m1, dir, g1, m2, g2)
    }
}