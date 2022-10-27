#[repr(C)]
pub struct Vert([f32; 2]);

pub fn make_ellipse(rx: f32, ry: f32) -> Vec<Vert> {
    use std::f32::consts::TAU;

    const MIN_SEGMENTS: u32 = 16;
    const SEGMENT_LEN: f32 = 6.;

    // Calculate the approximate number of points.
    // The circumference of an ellipse is quite complex,
    // so take the circumference of a circle
    let max_len = rx.max(ry) * TAU;
    let n = MIN_SEGMENTS.max((max_len / SEGMENT_LEN).ceil() as u32);
    println!("n = {n}");

    let step = TAU / n as f32;
    let scos = step.cos();
    let ssin = step.sin();
    let mut pcos = scos;
    let mut psin = -ssin;

    (0..n)
        .map(|_| {
            let x = pcos * scos - psin * ssin;
            let y = psin * scos + pcos * ssin;
            pcos = x;
            psin = y;
            Vert([x * rx, y * ry])
        })
        .collect()
}
