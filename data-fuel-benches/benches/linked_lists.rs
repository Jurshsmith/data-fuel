use criterion::{criterion_group, Criterion};
use linked_lists::{DoublyLinkedList, SinglyLinkedList};

pub fn singly_linked_list(c: &mut Criterion) {
    let mut list = SinglyLinkedList::new();
    c.bench_function("SinglyLinkedList Pushes", |b| {
        b.iter(|| {
            for i in 0..1_000 {
                list.push(i);
            }
        })
    });

    c.bench_function("SinglyLinkedList Iterations", |b| {
        b.iter(|| for _ in list.iter() {})
    });

    c.bench_function("SinglyLinkedList Pops", |b| {
        b.iter(|| {
            for _ in 0..1_000 {
                list.pop();
            }
        })
    });
}

pub fn doubly_linked_list(c: &mut Criterion) {
    let mut list = DoublyLinkedList::new();

    c.bench_function("DoublyLinkedList Pushes", |b| {
        b.iter(|| {
            for i in 0..1_000 {
                list.push(i);
            }
        })
    });

    c.bench_function("DoublyLinkedList Iterations", |b| {
        b.iter(|| for _ in list.iter() {})
    });

    c.bench_function("DoublyLinkedList Pops", |b| {
        b.iter(|| {
            for _ in 0..1_000 {
                list.pop();
            }
        })
    });
}

criterion_group!(linked_lists_benches, singly_linked_list, doubly_linked_list);
