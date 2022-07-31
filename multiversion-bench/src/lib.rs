use multiversion::multiversion;

pub fn square(i: &[f32], o: &mut [f32]) {
    for (i, o) in i.iter().zip(o) {
        *o = i * i;
    }
}

#[target_feature(enable = "avx")]
pub unsafe fn square_avx(i: &[f32], o: &mut [f32]) {
    for (i, o) in i.iter().zip(o) {
        *o = i * i;
    }
}

#[multiversion(targets("x86_64+avx"), dispatcher = "indirect")]
pub fn square_indirect(i: &[f32], o: &mut [f32]) {
    for (i, o) in i.iter().zip(o) {
        *o = i * i;
    }
}

#[multiversion(targets("x86_64+avx"), dispatcher = "direct")]
pub fn square_direct(i: &[f32], o: &mut [f32]) {
    for (i, o) in i.iter().zip(o) {
        *o = i * i;
    }
}
