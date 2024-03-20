use std::{cmp::Ordering, fmt::Debug};

#[derive(Debug, Clone)]
pub struct LexigraphicalCycle<T: Sized + Clone + Ord + Debug, const COUNT: usize> {
    cycle: [T; COUNT],
    rotation: usize,
}

impl<T: Sized + Clone + Ord + Debug, const SIZE: usize> PartialEq for LexigraphicalCycle<T, SIZE> {
    fn eq(&self, other: &Self) -> bool {
        self.cycle == other.cycle
    }
}

impl<T: Sized + Clone + Ord + Debug + Copy, const COUNT: usize> LexigraphicalCycle<T, COUNT>
where
    [T; COUNT]: Debug + for<'a> TryFrom<&'a [T]>,
{
    pub fn naive_minimal_rotation(arr: &[T; COUNT]) -> Self {
        let mut min_rotation_arr = *arr;
        let mut min_rotation = 0;

        for i in 0..arr.len() {
            let mut rotation = *arr;
            rotation.rotate_left(i);
            if rotation.iter().lt(min_rotation_arr.iter()) {
                min_rotation_arr = rotation;
                min_rotation = i;
            }
        }

        Self {
            cycle: min_rotation_arr,
            rotation: min_rotation,
        }
    }

    pub fn shiloah_minimal_rotation(arr: &[T; COUNT]) -> Self {
        let n = arr.len();

        let mut i = 0;
        let mut j = 1;
        let mut k = 0;

        while i < n && j < n && k < n {
            match arr[(i + k) % n].cmp(&arr[(j + k) % n]) {
                Ordering::Equal => {
                    k += 1;
                }
                Ordering::Greater => {
                    i += k + 1;

                    if i == j {
                        j += 1;
                    }
                    k = 0;
                }
                Ordering::Less => {
                    j += k + 1;

                    if i == j {
                        j += 1;
                    }
                    k = 0;
                }
            }
        }
        let rotation_point = i.min(j) % n;

        let mut res = *arr;
        res.rotate_left(rotation_point);

        LexigraphicalCycle {
            cycle: res,
            rotation: rotation_point,
        }
    }

    pub fn booth_minimal_rotation(arr: &[T; COUNT]) -> Self {
        let n2: i16 = 2 * COUNT as i16;
        let mut f: Vec<i16> = vec![-1; n2 as usize];
        let mut k: i16 = 0;
        for j in 1..n2 {
            let mut i: i16 = f[(j - k - 1) as usize];
            while i != -1 && arr[j as usize % COUNT] != arr[(k + i + 1) as usize % COUNT] {
                if arr[j as usize % COUNT] < arr[(k + i + 1) as usize % COUNT] {
                    k = j - i - 1;
                }
                i = f[i as usize];
            }
            if i == -1 && arr[j as usize % COUNT] != arr[(k + i + 1) as usize % COUNT] {
                if arr[j as usize % COUNT] < arr[(k + i + 1) as usize % COUNT] {
                    k = j;
                }
                f[(j - k) as usize] = -1;
            } else {
                f[(j - k) as usize] = i + 1;
            }
        }

        let mut res = *arr;
        res.rotate_left(k as usize);

        Self {
            cycle: res,
            rotation: k as usize,
        }
    }
}

#[cfg(test)]
mod tests {
    use self::test_utils::*;
    use crate::gameplay::map::utils::lexigraphical_cycle::LexigraphicalCycle;

    #[test]
    fn cycle_naive_rotation() {
        check_equality::<5>(LexigraphicalCycle::naive_minimal_rotation);
        check_correctness(LexigraphicalCycle::naive_minimal_rotation);
    }

    #[test]
    fn cycle_shiloah_rotation() {
        check_equality::<5>(LexigraphicalCycle::shiloah_minimal_rotation);
        check_correctness(LexigraphicalCycle::shiloah_minimal_rotation);
    }

    #[test]
    fn cycle_booth_rotation() {
        check_equality::<5>(LexigraphicalCycle::booth_minimal_rotation);
        check_correctness(LexigraphicalCycle::booth_minimal_rotation);
    }

    pub mod test_utils {
        use crate::gameplay::map::utils::lexigraphical_cycle::LexigraphicalCycle;
        use itertools::Itertools;

        pub fn check_equality<const SIZE: usize>(
            calculate: fn(&[i8; SIZE]) -> LexigraphicalCycle<i8, SIZE>,
        ) {
            let inputs = get_inputs::<SIZE>();

            let shifted_inputs = create_many_variations(inputs);
            for outputs in get_results(shifted_inputs, calculate) {
                for i in 0..outputs.len() {
                    for j in 0..outputs.len() {
                        assert_eq!(outputs[i], outputs[j]);
                    }
                }
            }
        }
        pub fn check_correctness(calculate: fn(&[i8; 7]) -> LexigraphicalCycle<i8, 7>) {
            let cycles = get_input_expected();
            for (input, expected) in cycles {
                let input_variations = create_variations(&input);
                for output in input_variations.iter().map(calculate) {
                    assert_eq!(output.cycle, expected);
                }
            }
        }
        fn get_results<const SIZE: usize>(
            shifted_inputs: Vec<Vec<[i8; SIZE]>>,
            calculate: fn(&[i8; SIZE]) -> LexigraphicalCycle<i8, SIZE>,
        ) -> Vec<Vec<LexigraphicalCycle<i8, SIZE>>> {
            shifted_inputs
                .iter()
                .map(move |inputs| inputs.iter().map(calculate).collect_vec())
                .collect_vec()
        }

        fn create_many_variations<const SIZE: usize>(
            inputs: Vec<[i8; SIZE]>,
        ) -> Vec<Vec<[i8; SIZE]>> {
            inputs
                .iter()
                .map(|i| {
                    let shifted_inputs: Vec<[i8; SIZE]> = create_variations(i);

                    shifted_inputs
                })
                .collect_vec()
        }

        fn create_variations<const SIZE: usize>(i: &[i8; SIZE]) -> Vec<[i8; SIZE]> {
            (0..i.len())
                .map(|shift| {
                    let mut input = *i;
                    input.rotate_right(shift);
                    input
                })
                .collect()
        }

        fn get_inputs<const SIZE: usize>() -> Vec<[i8; SIZE]> {
            let inputs: Vec<[i8; SIZE]> = (-2..3)
                .permutations(SIZE)
                .map(|i| i.try_into().unwrap())
                .collect();
            inputs
        }

        fn get_input_expected() -> [([i8; 7], [i8; 7]); 1] {
            [([3, 1, 3, 4, 5, 1, 7], [1, 3, 4, 5, 1, 7, 3])]
        }
    }
}
