use ggez::graphics;
use ggez::nalgebra as na;
use std::fmt;

pub fn str2col(s: &str) -> graphics::Color {
    let rgb = s
        .trim_start_matches('#')
        .chars()
        .collect::<Vec<char>>()
        .chunks(2)
        .map(|c| {
            u8::from_str_radix(c.iter().collect::<String>().as_str(), 16)
                .expect("Unable to parse color")
        })
        .collect::<Vec<u8>>();
    graphics::Color::from_rgb(rgb[0], rgb[1], rgb[2])
}

#[derive(Debug, PartialEq)]
pub struct ColoredTriangles {
    /// Colors that can be used to draw triangles
    pub colors: Vec<graphics::Color>,
    /// Vertices of triangles, one for each color
    pub triangles: Vec<Vec<na::Point2<f32>>>,
}

impl ColoredTriangles {
    pub fn add_color(&mut self, c: &str) {
        let color = str2col(c);
        if self
            .colors
            .iter()
            .enumerate()
            .find(|(_, c)| **c == color)
            .is_none()
        {
            self.colors.push(color);
            self.triangles.push(vec![]);
        }
    }

    /// Removes the 3 vertices corresponding to ith triangle for given color index c
    pub fn remove_triangle(&mut self, c: usize, i: usize) {
        let _ = self.triangles[c]
            .drain(i * 3..(i + 1) * 3)
            .collect::<Vec<_>>();
    }

    /// Indices of the n nearest points and their (squared) distances from point
    pub fn nearest(&self, n: usize, point: &na::Point2<f32>) -> Vec<(usize, usize, f32)> {
        // Compute distance from point to every point in triangles
        let mut distances: Vec<(usize, usize, f32)> = self
            .triangles
            .iter()
            .enumerate()
            .flat_map(|(c, v)| {
                v.iter()
                    .map(|p| na::distance_squared(p, point))
                    .enumerate()
                    .map(move |(i, d)| (c, i, d))
            })
            .collect::<Vec<_>>();
        // Sort distances
        distances
            .as_mut_slice()
            .sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

        distances.iter().take(n).cloned().collect()
    }

    /// Returns indices of the first triangle found containing the point, if any
    pub fn colliding(&self, point: &na::Point2<f32>) -> Option<(usize, usize)> {
        // https://stackoverflow.com/questions/2049582/how-to-determine-if-a-point-is-in-a-2d-triangle
        fn sign(p1: &na::Point2<f32>, p2: &na::Point2<f32>, p3: &na::Point2<f32>) -> f32 {
            (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
        }

        let contains = |p1: &na::Point2<f32>, p2: &na::Point2<f32>, p3: &na::Point2<f32>| {
            let d1 = sign(point, p1, p2);
            let d2 = sign(point, p2, p3);
            let d3 = sign(point, p3, p1);
            let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
            let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);
            !(has_neg && has_pos)
        };

        let collisions: Vec<(usize, usize)> = self
            .triangles
            .iter()
            .enumerate()
            .flat_map(|(c, v)| {
                v.as_slice()
                    .chunks(3)
                    .enumerate()
                    .filter_map(move |(i_tr, chunk)| {
                        if contains(&chunk[0], &chunk[1], &chunk[2]) {
                            Some((c, i_tr))
                        } else {
                            None
                        }
                    })
            })
            .collect();
        if collisions.is_empty() {
            None
        } else {
            Some(collisions[0])
        }
    }

    /// In place scale of triangles, multiplying each axis by a scale factor
    pub fn scale(&mut self, x_scale: f32, y_scale: f32) {
        for c in 0..self.triangles.len() {
            for i in 0..self.triangles[c].len() {
                self.triangles[c][i].x *= x_scale;
                self.triangles[c][i].y *= y_scale;
            }
        }
    }

    /// In place translation of triangles, adding to each axis an offset
    pub fn translate(&mut self, x_off: f32, y_off: f32) {
        for c in 0..self.triangles.len() {
            for i in 0..self.triangles[c].len() {
                self.triangles[c][i].x += x_off;
                self.triangles[c][i].y += y_off;
            }
        }
    }
}

impl From<&str> for ColoredTriangles {
    fn from(s: &str) -> Self {
        let mut ct = Self {
            colors: vec![],
            triangles: vec![],
        };
        s.lines().for_each(|l| {
            let mut data: Vec<_> = l.split_whitespace().collect();
            let col = data
                .remove(0)
                .parse::<u32>()
                .expect("Unable to parse color");
            let col = graphics::Color::from_rgb_u32(col);
            let mut coords = data
                .iter()
                .map(|tok| {
                    let p: Vec<f32> = tok.split(',').map(|v| v.parse::<f32>().unwrap()).collect();
                    na::Point2::new(p[0], p[1])
                })
                .collect::<Vec<_>>();
            // Find index of color into colors
            if let Some((c, _)) = ct.colors.iter().enumerate().find(|(_, c)| **c == col) {
                // Add triangles to color
                ct.triangles[c].append(&mut coords);
            } else {
                ct.colors.push(col);
                ct.triangles.push(coords);
            }
        });
        ct
    }
}

impl fmt::Display for ColoredTriangles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.colors.len() {
            writeln!(
                f,
                "{} {}",
                self.colors[i].to_rgb_u32(),
                self.triangles[i]
                    .iter()
                    .map(|p| format!("{},{}", p.x, p.y))
                    .collect::<Vec<String>>()
                    .join(" ")
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ColoredTriangles;
    use ggez::graphics;
    use ggez::nalgebra as na;

    #[test]
    fn colored_triangles_to_string() {
        let ct = ColoredTriangles {
            colors: vec![graphics::Color::from_rgb(0, 0, 255)],
            triangles: vec![vec![
                na::Point2::new(0.0, 0.0),
                na::Point2::new(1.0, 0.0),
                na::Point2::new(1.0, 1.0),
            ]],
        };
        assert_eq!(ct.to_string(), "255 0,0 1,0 1,1\n".to_owned());
    }

    #[test]
    fn colored_triangles_from_string() {
        let s: ColoredTriangles = "255 0.0,0.0 1.0,0.0 1.0,1.0\n".into();
        assert_eq!(
            s,
            ColoredTriangles {
                colors: vec![graphics::Color::from_rgb(0, 0, 255)],
                triangles: vec![vec![
                    na::Point2::new(0.0, 0.0),
                    na::Point2::new(1.0, 0.0),
                    na::Point2::new(1.0, 1.0),
                ]]
            }
        );
    }
}
