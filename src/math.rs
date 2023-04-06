#![allow(unused)]
use nalgebra::{DMatrix, Matrix3, Matrix3x1, Matrix6, OVector, SymmetricEigen, Vector3, U3};

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
) -> (Matrix3<f64>, Vector3<f64>) {
    let m_1 = m.try_inverse().expect("m to be inversible");
    let b = -(m_1 * n);
    let a_1 = (f / ((n.transpose() * (m_1 * n))[0] - d).sqrt()) * sqrt_m(&m);
    return (a_1, b);
}

/// Fits ellipsoid to set of points
pub fn ellipsoid_fit(s: &Vec<[f64; 3]>) -> (Matrix3<f64>, Vector3<f64>, f64) {
    let n = s.len();
    let mut d = DMatrix::<f64>::zeros(10, n);
    for j in 0..n {
        let s_j = OVector::<f64, U3>::from_row_slice(&s[j]);
        d[(0, j)] = s_j[0].powi(2);
        d[(1, j)] = s_j[1].powi(2);
        d[(2, j)] = s_j[2].powi(2);
        d[(3, j)] = 2.0 * s_j[1] * s_j[2];
        d[(4, j)] = 2.0 * s_j[0] * s_j[2];
        d[(5, j)] = 2.0 * s_j[0] * s_j[1];
        d[(6, j)] = 2.0 * s_j[0];
        d[(7, j)] = 2.0 * s_j[1];
        d[(8, j)] = 2.0 * s_j[2];
        d[(9, j)] = 1.0;
    }
    let s = d.clone() * d.transpose();
    let s_11 = s.fixed_view::<6, 6>(0, 0);
    let s_12 = s.fixed_view::<6, 4>(0, 6);
    let s_21 = s.fixed_view::<4, 6>(6, 0);
    let s_22 = s.fixed_view::<4, 4>(6, 6);
    let c = Matrix6::new(
        -1., 1., 1., 0., 0., 0., 1., -1., 1., 0., 0., 0., 1., 1., -1., 0., 0., 0., 0., 0., 0., -4.,
        0., 0., 0., 0., 0., 0., -4., 0., 0., 0., 0., 0., 0., -4.,
    );
    let inv_c = c.try_inverse().expect("C is not invertible");
    let inv_s_22 = s_22.try_inverse().expect("S_22 is not invertible");

    let e = inv_c * (s_11 - (s_12 * inv_s_22 * s_21));
    let eigendec = SymmetricEigen::new(e);
    let e_w = eigendec.eigenvalues;
    let e_v = eigendec.eigenvectors;

    let (argmax, _) = e_w.argmax();
    let p_v_1 = e_v.column(argmax);
    let v_1 = if p_v_1[0] < 0.0 {
        -1. * p_v_1
    } else {
        1. * p_v_1
    };

    let v_2 = (-inv_s_22 * s_21) * v_1;
    let m = Matrix3::new(
        v_1[0], v_1[3], v_1[4], v_1[3], v_1[1], v_1[5], v_1[4], v_1[5], v_1[2],
    );
    let n = Vector3::new(v_2[0], v_2[1], v_2[2]);
    let d = v_2[3];
    (m, n, d)
}

/// Transformation of a single sample
pub fn calibrated_sample(
    sample: &[f32; 3],
    a_1: &Matrix3<f32>,
    b: &Vector3<f32>,
) -> Matrix3x1<f32> {
    let s = Matrix3x1::from_row_slice(sample);
    let transformed_s = a_1 * (s - b);
    transformed_s
}
