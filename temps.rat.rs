let (a,b) = rtlsrc(434e6,256e3).abs().norm().trigger(.001).discretize().rle().dle(256e3).split()
let (c,d) = a.map(|x| {match x { (1, 2e-4..6e-4) => match a.next().unwrap() { (0, 1.5e-3..2.5e-3) => Some(0u), (0, 3.5e-3..4.5e-3) => Some(1u), _ => None}, _     => None}}).filter_agg(36).split();
let e = b.map(|x| {match x {(1, ref z@125e-6..250e-6) | (1, ref z @500e-6..650e-6) => {match b.next().unwrap() {(0, ref y @500e-6..650e-6) | (0, ref y@125e-6..250e-6    ) => Some(if z > y {1u} else {0u}),_ => None}}, _ => None}}).map(|x| {x.push(0);x});
let f = c.bin_slice([4,8,4,12,8]).filter(|x|x[0] ==5).vec_as_f32().el_mul(vec![1., 1., 1., 1e-1, 1.]);
let g = d.bin_slice([4,8,2,10,12]).filter(|x|x[0] ==5).vec_as_f32().el_mul(vec![1., 1., 1., 1e-1, 1e-1]);
let h = select_merge!(e,f,g).different_filter().map(|x| (time::get_time().sec, x));

for sample in k {
	println!("{}", sample);
}
