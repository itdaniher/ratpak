let (a,b) = rtlsrc(434e6,256e3).abs().norm().trigger(.001).discretize().rle().dle(256e3).split()
let (c,d) = a.map(|mut a,b| {a.map(|x| {b.send(match x { (1, 2e-4..6e-4) => match a.next().unwrap() { (0, 1.5e-3..2.5e-3) => Some(0u), (0, 3.5e-3..4.5e-3) => Some(1u), _ => None}, _     => None})})}).filter_agg(36).split();
let e = c.bin_slice([4,8,4,12,8]).filter(|x|x[0] ==5).vec_as_f32().el_mul(vec![1., 1., 1., 1e-1, 1.]);
let f = c.bin_slice([4,8,4,12,8]).filter(|x|x[0] ==5).vec_as_f32().el_mul(vec![1., 1., 1., 1e-1, 1e-1]);
let d = b.map(|mut a,b| {a.map(|x| {b.send(match x {(1, ref d@125e-6..250e-6) | (1, ref d @500e-6..650e-6) => {match a.next().unwrap() {(0, ref e @500e-6..650e-6) | (0, ref e@125e-6..250e-6    ) => Some(if d > e {1u} else {0u}),_ => None}}, _ => None})}).last();).map(|x| {x.push(0);x});
let e = select_merge!(e,f,d).different_filter().map(|x| (time::get_time().sec, x));

for sample in e {
	println!("{}", sample);
}
