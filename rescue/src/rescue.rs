use crate::inverse_sbox::InverseSboxLayer;
use crate::util::binomial;

use ethereum_types::U256;
use sha3::Shake256;
use std::marker::PhantomData;

use p3_field::PrimeField;
use p3_symmetric::permutation::MDSPermutation;
use p3_util::ceil_div_usize;

#[derive(Clone)]
pub struct Rescue<F, MDS, ISL, const WIDTH: usize, const CAPACITY: usize, const ALPHA: u64, const SEC_LEVEL: usize>
where
    F: PrimeField,
    MDS: MDSPermutation<F, WIDTH>,
    ISL: InverseSboxLayer<F, WIDTH, ALPHA>,
{
    num_rounds: usize,
    mds: MDS,
    rate: usize,
    round_constants: Vec<F>,

    _phantom_f: PhantomData<F>,
    _phantom_isl: PhantomData<ISL>,
}

impl<F, MDS, ISL, const WIDTH: usize, const CAPACITY: usize, const ALPHA: u64, const SEC_LEVEL: usize>
    Rescue<F, MDS, ISL, WIDTH, CAPACITY, ALPHA, SEC_LEVEL>
where
    F: PrimeField,
    MDS: MDSPermutation<F, WIDTH>,
    ISL: InverseSboxLayer<F, WIDTH, ALPHA>,
{
    pub fn new(num_rounds: usize, round_constants: Vec<F>, mds: MDS) -> Self {
        Self {
            num_rounds,
            mds,
            rate: WIDTH - CAPACITY,
            round_constants,
            _phantom_f: PhantomData,
            _phantom_isl: PhantomData,
        }
    }

    fn num_rounds() -> usize {
        let rate = WIDTH - CAPACITY;
        let dcon = |n: usize| (0.5 * ((ALPHA-1) * WIDTH as u64 * (n as u64 - 1)) as f64 + 2.0).floor() as usize;
        let v = |n: usize| WIDTH * (n - 1) + rate;
        let target = U256::one() << SEC_LEVEL;

        let is_sufficient = |l1: &usize| {
            let bin = binomial(v(*l1) + dcon(*l1), v(*l1));
            bin * bin > target
        };
        (1..25).find(is_sufficient).unwrap()
    }

    fn sbox_layer(state: &mut [F; WIDTH]) {
        for x in state.iter_mut() {
            *x = x.exp_u64(ALPHA);
        }
    }

    fn rescue_XLIX_permutation(&self, state: &mut [F; WIDTH]) {
        for round in 0..self.num_rounds {
            // S-box
            Self::sbox_layer(state);

            // MDS
            self.mds.permute_mut(state);

            // Constants
            for j in 0..WIDTH {
                state[j] += self.round_constants[round * WIDTH * 2 + j];
            }

            // Inverse S-box
            ISL::inverse_sbox_layer(state);

            // MDS
            self.mds.permute_mut(state);

            // Constants
            for j in 0..WIDTH {
                state[j] += self.round_constants[round * WIDTH * 2 + WIDTH + j];
            }
        }
    }

    // fn get_round_constants() -> Vec<F> {
    //     let bytes_per_int = ceil_div_usize(F::BITS, 8) + 1;
    //     let num_bytes = bytes_per_int * 2 * WIDTH * self.num_rounds;
    //     let seed_string = format!("Rescue-XLIX({},{},{},{}", F::order(), WIDTH, CAPACITY, SEC_LEVEL);

    //     let mut hasher = Shake256::new();
    //     hasher.update(seed_string.as_bytes());
    //     let byte_string = hasher.finalize()[..];
    // }
}




// type RescuePrimeOptimizedM31 = Rescue<Mersenne31, ... >
// fn new_rescue_prime_optimized_m31() -> RescuePrimeOptimizedM31 {