use crate::{
    app::App,
    constant::{BASE_BITS_NUM, NOTE_COMMIT_DOMAIN},
    nullifier::Nullifier,
    user::User,
    utils::extract_p,
};
use bitvec::{array::BitArray, order::Lsb0};
use core::iter;
use ff::{Field, PrimeFieldBits};
use group::Group;
use pasta_curves::pallas;
use rand::{Rng, RngCore};

/// A commitment to a note.
#[derive(Copy, Debug, Clone)]
pub struct NoteCommitment(pallas::Point);

impl NoteCommitment {
    pub fn inner(&self) -> pallas::Point {
        self.0
    }

    pub fn get_x(&self) -> pallas::Base {
        extract_p(&self.0)
    }
}

impl Default for NoteCommitment {
    fn default() -> NoteCommitment {
        NoteCommitment(pallas::Point::generator())
    }
}

/// A note
#[derive(Debug, Clone, Default)]
pub struct Note {
    /// Owner of the note
    pub user: User,
    pub app: App,
    pub value: u64,
    /// for NFT or whatever. TODO: to be decided the value format.
    pub data: pallas::Base,
    /// old nullifier. Nonce which is a deterministically computed, unique nonce
    pub rho: Nullifier,
    /// computed from spent_note_nf and rcm by using a PRF
    pub psi: pallas::Base,
    pub rcm: pallas::Scalar,
    /// The non-normal note won't be stored in commitment tree.
    pub is_normal: bool,
}

impl Note {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user: User,
        app: App,
        value: u64,
        rho: Nullifier,
        data: pallas::Base,
        psi: pallas::Base,
        rcm: pallas::Scalar,
        is_normal: bool,
    ) -> Self {
        Self {
            user,
            app,
            value,
            data,
            rho,
            psi,
            rcm,
            is_normal,
        }
    }

    pub fn dummy<R: RngCore>(mut rng: R) -> Self {
        let rho = Nullifier::new(pallas::Base::random(&mut rng));
        Self::dummy_from_rho(rng, rho)
    }

    pub fn dummy_from_rho<R: RngCore>(mut rng: R, rho: Nullifier) -> Self {
        let user = User::dummy(&mut rng);
        let app = App::dummy(&mut rng);
        let value: u64 = rng.gen();
        let data = pallas::Base::random(&mut rng);
        let rcm = pallas::Scalar::random(&mut rng);
        let psi = pallas::Base::random(&mut rng);
        Self {
            user,
            app,
            value,
            data,
            rho,
            psi,
            rcm,
            is_normal: true,
        }
    }

    // cm = SinsemillaCommit^rcm(user_address || app_address || data || rho || psi || is_normal || value)
    pub fn commitment(&self) -> NoteCommitment {
        let user_address = self.user.address();
        let app_address = self.app.address();
        let ret = NOTE_COMMIT_DOMAIN
            .commit(
                iter::empty()
                    .chain(
                        user_address
                            .to_le_bits()
                            .iter()
                            .by_vals()
                            .take(BASE_BITS_NUM),
                    )
                    .chain(
                        app_address
                            .to_le_bits()
                            .iter()
                            .by_vals()
                            .take(BASE_BITS_NUM),
                    )
                    .chain(self.data.to_le_bits().iter().by_vals().take(BASE_BITS_NUM))
                    .chain(
                        self.rho
                            .inner()
                            .to_le_bits()
                            .iter()
                            .by_vals()
                            .take(BASE_BITS_NUM),
                    )
                    .chain(self.psi.to_le_bits().iter().by_vals().take(BASE_BITS_NUM))
                    .chain([self.is_normal])
                    .chain(
                        BitArray::<_, Lsb0>::new(self.value.to_le())
                            .iter()
                            .by_vals(),
                    ),
                &self.rcm,
            )
            .unwrap();
        NoteCommitment(ret)
    }

    pub fn get_nf(&self) -> Nullifier {
        let nk = self.user.get_nk().unwrap();
        let cm = self.commitment();
        Nullifier::derive_native(&nk, &self.rho.inner(), &self.psi, &cm)
    }
}
