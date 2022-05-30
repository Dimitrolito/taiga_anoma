use ark_bls12_377::Fr as Fr377;
use ark_ec::twisted_edwards_extended::GroupAffine as TEGroupAffine;
use ark_ec::{AffineCurve, TEModelParameters};
use ark_ff::*;
use ark_serialize::CanonicalSerialize;
use circuit::circuit_parameters::CircuitParameters;
use rs_merkle::{algorithms::Blake2s, Hasher, MerkleTree};
use sha2::{Digest, Sha512};

pub mod action;
pub mod circuit;
pub mod el_gamal;
pub mod note;
pub mod token;
pub mod transaction;
pub mod user;

pub trait HashToField: PrimeField {
    fn hash_to_field(x: &[u8]) -> Self {
        // that's not a good hash but I don't care
        let mut hasher = Sha512::new();
        hasher.update(x);
        let result = hasher.finalize();
        Self::from_le_bytes_mod_order(&result)
    }
}

impl HashToField for ark_vesta::Fr {}
impl HashToField for ark_vesta::Fq {}
impl HashToField for ark_bls12_377::Fq {}
// impl HashToField for ark_bw6_761::Fq {} // even sha512 is not enough...
impl HashToField for ark_cp6_782::Fq {} // even sha512 is not enough...
impl HashToField for ark_ed_on_bls12_377::Fr {}

impl HashToField for Fr377 {
    fn hash_to_field(x: &[u8]) -> Self {
        // poseidon implementation
        // todo what is the domain separator?
        // implementation not working for a large input `x`...

        let elts: Vec<Fr377> = x
            .chunks((Fr377::size_in_bits() - 1) / 8 as usize)
            .map(|elt| Fr377::from_le_bytes_mod_order(elt))
            .collect();

        assert!(elts.len() <= 5);
        match elts.len() {
            1 => poseidon377::hash_1(&Fr377::zero(), elts[0]),
            2 => poseidon377::hash_2(&Fr377::zero(), (elts[0], elts[1])),
            3 => poseidon377::hash_3(&Fr377::zero(), (elts[0], elts[1], elts[2])),
            4 => poseidon377::hash_4(&Fr377::zero(), (elts[0], elts[1], elts[2], elts[3])),
            _ => poseidon377::hash_5(
                &Fr377::zero(),
                (elts[0], elts[1], elts[2], elts[3], elts[4]),
            ),
        }
    }
}

/// Pseudorandom function
fn prf<F: PrimeField + HashToField>(x: &[u8]) -> F {
    F::hash_to_field(x)
}

fn com<F: PrimeField + HashToField>(x: &[u8], rand: BigInteger256) -> F {
    // F is supposed to be CurveBaseField
    let y = rand.to_bytes_le();
    let z = [x, &y].concat();
    F::hash_to_field(&z)
}

fn serializable_to_vec<F: CanonicalSerialize>(elem: &F) -> Vec<u8> {
    let mut bytes_prep_send = vec![];
    elem.serialize_unchecked(&mut bytes_prep_send).unwrap();
    bytes_prep_send
}

// A really bad hash-to-curve
// TODO: the implementation is a bit weird: it does not really depends on CP and could be written with a curve as a parameter (`fn hash_to_curve<E:Curve>`).
fn hash_to_curve<CP: CircuitParameters>(
    data: &[u8],
    rand: BigInteger256, // TODO: Rand is not used!
) -> TEGroupAffine<CP::InnerCurve> {
    let scalar = <CP::InnerCurveScalarField>::hash_to_field(data);
    TEGroupAffine::prime_subgroup_generator().mul(scalar).into()
}


fn add_to_tree<P: TEModelParameters>(elem: &TEGroupAffine<P>, tree: &mut MerkleTree<Blake2s>) {
    let bytes = serializable_to_vec(elem);
    let h = Blake2s::hash(&bytes);
    tree.insert(h);
    tree.commit();
}


#[cfg(test)]
pub mod tests;
