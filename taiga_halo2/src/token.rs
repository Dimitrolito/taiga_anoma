use crate::vp_description::ValidityPredicateDescription;
use pasta_curves::pallas;
use rand::RngCore;

#[derive(Debug, Clone)]
pub struct Token {
    pub token_vp: ValidityPredicateDescription,
}

impl Token {
    pub fn new(token_vp_description: ValidityPredicateDescription) -> Self {
        Self {
            token_vp: token_vp_description,
        }
    }

    pub fn dummy(rng: &mut impl RngCore) -> Self {
        Self {
            token_vp: ValidityPredicateDescription::dummy(rng),
        }
    }

    pub fn address(&self) -> pallas::Base {
        self.token_vp.get_compressed()
    }
}

impl Default for Token {
    fn default() -> Token {
        let token_vp = ValidityPredicateDescription::Compressed(pallas::Base::one());
        Token { token_vp }
    }
}
