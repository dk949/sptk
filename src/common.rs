macro_rules! iota {
    ($l:literal) => {iota![usize;$l]};
    ($t:ty; $l:literal) => {{
        let mut v = [0; $l];
        for (n, a) in v.iter_mut().enumerate() {
            *a = n as $t;
        }
        v
    }};
    ($e:expr) => {iota![usize;$e]};
    ($t:ty; $e:expr) => {{
        let mut v = vec![0; $e];
        for (n, a) in v.iter_mut().enumerate() {
            *a = n as $t;
        }
        v
    }};
}

pub(crate) use iota;
