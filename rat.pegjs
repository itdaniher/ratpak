start =
   n: network+ "\n\n" { return n }

name =
  proc: ($[a-zA-Z"]+)

value =
   [a-zA-Z0-9, ']+

args = " "? "(" v:$value+ ")" " "? { return v}

dyad = " "? op:[v^] " "? {return op}

proc =
	n: name a: args? { return [n, a] }

fbrk =
	d: dyad rp: proc { x = [d]; x.push(rp); return x }

expr =
  lp: proc d:fbrk* {var x = [lp]; x.push(d); return x}

line =
  "\t" " "* e:((lp:"("? ex:expr rp:")"? sp:" "?){return ex})+"\n" {return e}

network =
   i:"=>"? " "? id:name " "? o:"=>"? "\n" lines:line+ "\n" {return [id, i == "=>", o == "=>", lines]}
