pub fn normalize(q: &mut [f32; 4]) {
    let epsilon = 0.00001;
    let mut d = 0.0;

    for i in 0..4 {
        d += q[i].powi(2);
    }

    if d < epsilon {
        return; // do nothing if it is zero
    }

    let inv_length = 1.0 / d.sqrt();
    q[0] *= inv_length;
    q[1] *= inv_length;
    q[2] *= inv_length;
    q[3] *= inv_length;  
}

pub fn slerp(q1: &[f32; 4], q2: &[f32; 4], max_angle: f32) -> [f32; 4] {
    
    if max_angle < 0.001 {
		// No rotation allowed. Prevent dividing by 0 later.
        return *q1;
    }

    let mut cos_theta = 0.0;
    for i in 0..4 {
        cos_theta += q1[i] * q2[i];
    }

	// q1 and q2 are already equal.
	// Force q2 just to be sure
    if cos_theta > 0.9999 {
        return *q2;
    }

    let mut res = [0.0; 4];

	// Avoid taking the long path around the sphere
    if cos_theta < 0.0 {
        for i in 0..4 {
            res[i] = q1[i] * -1.0;
        }
        cos_theta *= -1.0;
    }
    else {
        for i in 0..4 {
            res[i] = q1[i];
        }
    }

    let angle = cos_theta.acos();

	// If there is only a 2&deg; difference, and we are allowed 5&deg;,
	// ten we arrive:
    if angle < max_angle {
        return *q2;
    }

    let ft = max_angle / angle;
    let angle = max_angle;

    let a = ((1.0 - ft) * angle).sin();
    let b = (ft * angle).sin();
    let _sin = angle.sin();
    for i in 0..4 {
        res[i] = (a * res[i] + b * q2[i]) / _sin;
    }
    normalize(&mut res);
    return res
}