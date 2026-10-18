use super::reserve_bounded;

#[test]
fn reservation_admits_only_remaining_elements_without_changing_contents() {
    let mut items = Vec::new();
    assert_eq!(reserve_bounded(&mut items, usize::MAX, 0).unwrap(), 0);
    assert_eq!(items.capacity(), 0);
    for additional in [1, 3, 5, usize::MAX] {
        let before = items.clone();
        let admitted = reserve_bounded(&mut items, additional, 11).unwrap();
        assert_eq!(items, before);
        assert_eq!(admitted, additional.min(11 - items.len()));
        assert!(items.capacity() >= items.len() + admitted);
        items.extend(std::iter::repeat_n(7_u8, admitted));
    }
    assert_eq!(items.len(), 11);
    let capacity = items.capacity();
    assert_eq!(reserve_bounded(&mut items, usize::MAX, 3).unwrap(), 0);
    assert_eq!(items, vec![7; 11]);
    assert_eq!(items.capacity(), capacity);

    // The allowance counts elements, including zero-sized elements.
    let mut empty_elements = vec![(); 2];
    assert_eq!(reserve_bounded(&mut empty_elements, 5, 4).unwrap(), 2);
    assert_eq!(empty_elements.len(), 2);
}

#[test]
fn reservation_returns_capacity_overflow_without_changing_contents() {
    let mut items = vec![1_u8];
    assert!(reserve_bounded(&mut items, usize::MAX, usize::MAX).is_err());
    assert_eq!(items, [1]);
}
