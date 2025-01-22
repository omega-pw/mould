/**
 * 按交集来分区
 */
pub fn partition_by_intersection<T1, T2>(
    list1: Vec<T1>,
    list2: &[T2],
    eq: impl Fn(&T1, &T2) -> bool,
) -> (Vec<T1>, Vec<T1>) {
    return list1
        .into_iter()
        .partition(|v1| list2.iter().any(|v2| eq(v1, v2)));
}

/**
 * 求交集
 */
pub fn intersect<'a, T1, T2>(
    list1: &'a [T1],
    list2: &[T2],
    eq: impl Fn(&T1, &T2) -> bool,
) -> Vec<&'a T1> {
    return list1
        .into_iter()
        .filter(|v1| list2.iter().any(|v2| eq(v1, v2)))
        .collect();
}
