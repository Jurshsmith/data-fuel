mod linked_lists;

use criterion::criterion_main;
use linked_lists::linked_lists_benches;

criterion_main!(linked_lists_benches);
