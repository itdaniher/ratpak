{
	var _ = require("underscore");
}

start =
   n: network+ "\n\n" { return n }

name =
  (portal: "."? proc: $[\+\*\&\>\<\-\/a-zA-Z0-9"]+ ) { if (portal) {return portal+proc} if (!portal) {return proc} }

value =
   ("0."?[a-zA-Z0-9, '])+

args = " "? "(" v:$value+ ")" " "? { return v}

dyad = " "? op:[v^] " "? {return op}

proc =
	n: name a: args? { if (a) { return {"proc": n, "args": a} } else { return {"proc": n} } }

fbrk =
	d: dyad rp: proc { return _.extend({"dyad": d}, rp)}

expr =
  lp: proc d:fbrk* {if (d) { return [lp,d] } else { return lp }}

line =
  "\t" " "* e:((lp:"("? ex:expr rp:")"? sp:" "?){return ex})+"\n" {return {"line":_.flatten(e)}}

network =
   i:"=>"? " "? id:name " "? o:"=>"? "\n" lines:line* "\n" {return {"name": id, "in":(i == "=>"), "out": (o == "=>"), "lines": lines}}
