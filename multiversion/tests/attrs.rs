#[multiversion::multiversion(targets("x86_64+avx"), attrs(track_caller, inline(never)))]
#[cfg(all())] // this attribute should only be attached to the multiversioned `inner_attrs` function
fn inner_attrs() {}
