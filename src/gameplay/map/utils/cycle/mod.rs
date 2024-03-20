use std::{cmp::Ordering, fmt::Debug};

#[derive(Debug, Clone)]
pub struct Cycle<T: Sized + Clone + Ord + Debug, const COUNT: usize> {
    cycle: [T; COUNT],
    rotation: usize,
}

impl<T: Sized + Clone + Ord + Debug, const COUNT: usize> Cycle<T, COUNT>
where
    [T; COUNT]: Debug + for<'a> TryFrom<&'a [T]>,
{
    pub fn naive_minimal_rotation(arr: &[T; COUNT]) -> Self {
        let mut min_rotation_arr = arr.clone();
        let mut min_rotation = 0;

        for i in 0..arr.len() {
            let mut rotation = arr.clone();
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
        let doubled_arr = [arr.to_vec(), arr.to_vec()].concat();
        let n = doubled_arr.len();

        let mut i = 0;
        let mut j = 1;
        let mut k = 0;

        while i < n && j < n && k < n {
            let comp = doubled_arr[(i + k) % n].cmp(&doubled_arr[(j + k) % n]);
            if comp == Ordering::Equal {
                k += 1;
            } else {
                if comp == Ordering::Greater {
                    i += k + 1;
                } else {
                    j += k + 1;
                }
                if i == j {
                    j += 1;
                }
                k = 0;
            }
        }
        let rotation_point = i.min(j);

        let minimal_rotation = &doubled_arr[rotation_point..rotation_point + n / 2];
        let res = &minimal_rotation[..arr.len()];

        Cycle {
            cycle: res.to_vec().try_into().unwrap(),
            rotation: rotation_point,
        }
    }

    //     let mut i = 0;
    //     let mut j = 1;
    //     let mut k = 0;

    //     while i < n && j < n && k < n {
    //         match doubled_arr[(i + k) % n].cmp(&doubled_arr[(j + k) % n]) {
    //             Ordering::Equal => k += 1,
    //             Ordering::Greater => {
    //                 i += k + 1;
    //                 if i == j {
    //                     j += 1;
    //                 }
    //             }
    //             Ordering::Less => {
    //                 j += k + 1;
    //                 if i == j {
    //                     j += 1;
    //                 }
    //             }
    //         }
    //     }
    //     let min_rotation_index = i.min(j);

    //     let cycle = match doubled_arr[min_rotation_index..min_rotation_index + n / 2][..arr.len()]
    //         .try_into()
    //     {
    //         Ok(res) => res,
    //         Err(_) => panic!("Failed converting cycle {:?} to sized arr", arr),
    //     };

    //     Cycle {
    //         cycle,
    //         rotation: min_rotation_index,
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use crate::gameplay::map::utils::cycle::Cycle;

    #[test]
    fn cycle_naive_rotation() {
        let cycles = [([3, 1, 3, 4, 5, 1, 7], [1, 3, 4, 5, 1, 7, 3])];
        for (cycle, expected) in cycles {
            for shift in (0..cycle.len()) {
                let mut input = cycle.clone();
                input.rotate_right(shift);

                let res = Cycle::naive_minimal_rotation(&input);
                assert!(
                    res.cycle.iter().eq(expected.iter()),
                    "{:?} != {:?}",
                    res,
                    expected
                );

                let mut rotated = input.clone();
                rotated.rotate_left(res.rotation);

                assert!(
                    rotated.iter().eq(expected.iter()),
                    "{:?} != {:?}",
                    rotated,
                    expected
                );
            }
        }
    }

    #[test]
    fn shiloah_rotation() {
        let cycles = [([3, 1, 3, 4, 5, 1, 7], [1, 3, 4, 5, 1, 7, 3])];
        for (cycle, expected) in cycles {
            for shift in (0..cycle.len()) {
                let mut input = cycle.clone();
                input.rotate_right(shift);

                let res = Cycle::shiloah_minimal_rotation(&input);
                assert!(
                    res.cycle.iter().eq(expected.iter()),
                    "{:?} != {:?}",
                    res,
                    expected
                );

                let mut rotated = input.clone();
                rotated.rotate_left(res.rotation);

                assert!(
                    rotated.iter().eq(expected.iter()),
                    "{:?} != {:?}",
                    rotated,
                    expected
                );
            }
        }
    }
}
