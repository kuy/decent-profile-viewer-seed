pub fn scale(domain: (f64, f64), codomain: (f64, f64)) -> impl Fn(f64) -> f64 {
  let (input_min, input_max) = domain;
  assert!(input_min <= input_max, "{} <= {}", input_min, input_max);

  let mut inverted = false;
  let (mut output_min, mut output_max) = codomain;
  if output_max < output_min {
    output_min = codomain.1;
    output_max = codomain.0;
    inverted = true;
  }

  move |input| {
    if input <= input_min {
      return if inverted { output_max } else { output_min };
    }

    if input_max <= input {
      return if inverted { output_min } else { output_max };
    }

    let ratio = (input - input_min) / (input_max - input_min);
    if inverted {
      output_max - (output_max - output_min) * ratio
    } else {
      (output_max - output_min) * ratio + output_min
    }
  }
}

#[cfg(test)]
mod tests {
  use nom::error::{Error, ErrorKind};

  use super::*;

  #[test]
  fn test_scale() {
    let x = scale((0., 100.), (100., 400.));
    assert_eq!(x(0.), 100.);
    assert_eq!(x(100.), 400.);

    assert_eq!(x(25.), 175.);
    assert_eq!(x(50.), 250.);
    assert_eq!(x(75.), 325.);

    assert_eq!(x(-1.2), 100.);
    assert_eq!(x(101.), 400.);
  }

  #[test]
  fn test_scale_inverted() {
    let y = scale((0., 100.), (370., 20.));
    assert_eq!(y(0.), 370.);
    assert_eq!(y(100.), 20.);

    assert_eq!(y(25.), 282.5);
    assert_eq!(y(50.), 195.);
    assert_eq!(y(75.), 107.5);

    assert_eq!(y(-10.), 370.);
    assert_eq!(y(100.5), 20.);
  }
}
