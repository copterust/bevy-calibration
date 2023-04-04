#![allow(unused)]
use nalgebra::{Matrix3, Vector3};

/// Returns principal square root of the 3x3 matrix
fn sqrt_m(matrix: &Matrix3<f64>) -> Matrix3<f64> {
    let e = matrix.symmetric_eigen();
    let d = e.eigenvalues;
    let q = e.eigenvectors;
    let mut sqrt_d = Vector3::new(0.0, 0.0, 0.0);
    for i in 0..3 {
        sqrt_d[i] = d[i].sqrt();
    }
    let d_sqrt = Matrix3::from_diagonal(&sqrt_d);
    q * d_sqrt * q.try_inverse().unwrap()
}

/// Returns a_1 and b to be applied to raw sensor data
pub fn ellipsoid_to_calibration(
    m: Matrix3<f64>,
    n: Vector3<f64>,
    d: f64,
    f: f64,
) -> (Matrix3<f64>, f64) {
    let m_1 = m.try_inverse().expect("m to be inversible");
    let b = n.dot(&(m_1 * n));
    let a_1 = (f / (b - d).sqrt()) * sqrt_m(&m);
    return (a_1, b);
}
