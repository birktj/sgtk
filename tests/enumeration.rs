use sgtk::*;


#[test]
fn enumerate_all_graphs5() {
    let mut enumerator = enumeration::Enumerator16::new(5);
    enumerator.enumerate();

    assert_eq!(enumerator.graphs.len(), 34);
}

#[test]
fn enumerate_all_graphs6() {
    let mut enumerator = enumeration::Enumerator16::new(6);
    enumerator.enumerate();

    assert_eq!(enumerator.graphs.len(), 156);
}

#[test]
fn enumerate_all_graphs7() {
    let mut enumerator = enumeration::Enumerator16::new(7);
    enumerator.enumerate();

    assert_eq!(enumerator.graphs.len(), 1044);
}

#[test]
fn enumerate_all_graphs8() {
    let mut enumerator = enumeration::Enumerator16::new(8);
    enumerator.enumerate();

    assert_eq!(enumerator.graphs.len(), 12346);
}
