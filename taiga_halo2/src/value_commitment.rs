use crate::constant::NOTE_COMMITMENT_R_GENERATOR;
use crate::note::Note;
use group::{cofactor::CofactorCurveAffine, Curve, Group};
use halo2_proofs::arithmetic::CurveAffine;
use pasta_curves::pallas;

#[derive(Copy, Clone, Debug)]
pub struct ValueCommitment(pallas::Point);

impl ValueCommitment {
    pub fn new(input_note: &Note, output_note: &Note, blind_r: &pallas::Scalar) -> Self {
        let base_input = input_note.derivate_value_base();
        let base_output = output_note.derivate_value_base();
        ValueCommitment(
            base_input * pallas::Scalar::from(input_note.value)
                - base_output * pallas::Scalar::from(output_note.value)
                + NOTE_COMMITMENT_R_GENERATOR.to_curve() * blind_r,
        )
    }

    pub fn get_x(&self) -> pallas::Base {
        if self.0 == pallas::Point::identity() {
            pallas::Base::zero()
        } else {
            *self.0.to_affine().coordinates().unwrap().x()
        }
    }

    pub fn get_y(&self) -> pallas::Base {
        if self.0 == pallas::Point::identity() {
            pallas::Base::zero()
        } else {
            *self.0.to_affine().coordinates().unwrap().y()
        }
    }
}
