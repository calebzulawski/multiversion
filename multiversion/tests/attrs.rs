#[multiversion::multiversion(targets("x86_64+avx"), attrs(track_caller, inline(never)))]
#[allow(dead_code)] // this attribute should only be attached to the multiversioned `inner_attrs` function, and none of the function clones
fn inner_attrs() {}
