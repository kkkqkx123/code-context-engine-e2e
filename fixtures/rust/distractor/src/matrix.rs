use std::ops::{Add, Mul};

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<T> {
    rows: usize,
    cols: usize,
    data: Vec<T>,
}

impl<T> Matrix<T> {
    pub fn new(rows: usize, cols: usize, data: Vec<T>) -> Self {
        assert_eq!(data.len(), rows * cols);
        Matrix { rows, cols, data }
    }

    pub fn from_vec(rows: usize, cols: usize, data: Vec<T>) -> Self {
        Self::new(rows, cols, data)
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn get(&self, r: usize, c: usize) -> &T {
        &self.data[r * self.cols + c]
    }

    pub fn get_mut(&mut self, r: usize, c: usize) -> &mut T {
        &mut self.data[r * self.cols + c]
    }
}

impl Matrix<f64> {
    pub fn identity(n: usize) -> Self {
        let mut data = vec![0.0; n * n];
        for i in 0..n {
            data[i * n + i] = 1.0;
        }
        Matrix::new(n, n, data)
    }

    pub fn zero(rows: usize, cols: usize) -> Self {
        Matrix::new(rows, cols, vec![0.0; rows * cols])
    }

    pub fn transpose(&self) -> Matrix<f64> {
        let mut data = vec![0.0; self.rows * self.cols];
        for r in 0..self.rows {
            for c in 0..self.cols {
                data[c * self.rows + r] = self.data[r * self.cols + c];
            }
        }
        Matrix::new(self.cols, self.rows, data)
    }

    pub fn determinant(&self) -> f64 {
        assert_eq!(self.rows, self.cols);
        let n = self.rows;
        if n == 1 {
            return self.data[0];
        }
        if n == 2 {
            return self.data[0] * self.data[3] - self.data[1] * self.data[2];
        }
        let mut det = 0.0;
        for c in 0..n {
            let sign = if c % 2 == 0 { 1.0 } else { -1.0 };
            det += sign * self.data[c] * self.minor(0, c).determinant();
        }
        det
    }

    fn minor(&self, row: usize, col: usize) -> Matrix<f64> {
        let n = self.rows;
        let mut data = Vec::with_capacity((n - 1) * (n - 1));
        for r in 0..n {
            if r == row {
                continue;
            }
            for c in 0..n {
                if c == col {
                    continue;
                }
                data.push(self.data[r * n + c]);
            }
        }
        Matrix::new(n - 1, n - 1, data)
    }
}

impl<T: Add<Output = T> + Clone> Add for &Matrix<T> {
    type Output = Matrix<T>;

    fn add(self, other: &Matrix<T>) -> Matrix<T> {
        assert_eq!(self.rows, other.rows);
        assert_eq!(self.cols, other.cols);
        let data: Vec<T> = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a.clone() + b.clone())
            .collect();
        Matrix::new(self.rows, self.cols, data)
    }
}

impl Mul<f64> for &Matrix<f64> {
    type Output = Matrix<f64>;

    fn mul(self, scalar: f64) -> Matrix<f64> {
        let data: Vec<f64> = self.data.iter().map(|x| x * scalar).collect();
        Matrix::new(self.rows, self.cols, data)
    }
}

impl Mul for &Matrix<f64> {
    type Output = Matrix<f64>;

    fn mul(self, other: &Matrix<f64>) -> Matrix<f64> {
        assert_eq!(self.cols, other.rows);
        let mut data = vec![0.0; self.rows * other.cols];
        for r in 0..self.rows {
            for c in 0..other.cols {
                let mut sum = 0.0;
                for k in 0..self.cols {
                    sum += self.data[r * self.cols + k] * other.data[k * other.cols + c];
                }
                data[r * other.cols + c] = sum;
            }
        }
        Matrix::new(self.rows, other.cols, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_addition() {
        let a = Matrix::new(2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        let b = Matrix::new(2, 2, vec![5.0, 6.0, 7.0, 8.0]);
        let c = &a + &b;
        assert_eq!(c.get(0, 0), &6.0);
        assert_eq!(c.get(1, 1), &12.0);
    }

    #[test]
    fn test_matrix_multiplication() {
        let a = Matrix::new(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let b = Matrix::new(3, 2, vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0]);
        let c = &a * &b;
        assert_eq!(c.rows(), 2);
        assert_eq!(c.cols(), 2);
    }

    #[test]
    fn test_determinant() {
        let m = Matrix::new(3, 3, vec![6.0, 1.0, 1.0, 4.0, -2.0, 5.0, 2.0, 8.0, 7.0]);
        let det = m.determinant();
        assert!((det - (-306.0)).abs() < 1e-10);
    }

    #[test]
    fn test_transpose() {
        let m = Matrix::new(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let t = m.transpose();
        assert_eq!(t.rows(), 3);
        assert_eq!(t.cols(), 2);
        assert_eq!(t.get(0, 1), &4.0);
    }
}
