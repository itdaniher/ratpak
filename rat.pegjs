{
	var _ = require("underscore");
}

start =
	n: network+ "\n\n" { return n }

name =
	$([+\*\&\>\<\-\/a-zA-Z"]+[0-9]*)

value =
	$("-"?"0."?[a-zA-Z0-9']+","?" "?)+

ref =
	" "? "(" " "?r:ref " "? ")" " "? {return r}
	/ r:$("." "-"? [0-9]+) {return r}

args = " "? "(" v:$value ")" " "? { return v}

dyad = " "? op:[v^] " "? {return op}

proc =
	n: name a: args { return {"proc": n, "args": a} }
	/ n: name r: ref+ {return {"proc":n, "refs": r} }
	/ r: ref {return {"ref": r}}
	/ n: name {return {"proc": n}}

expr =
	d: dyad rp: proc {return _.extend(rp, {"dyad": d})}
	/ proc

line =
	"\t" " "* e:((lp:"("? ex:expr rp:")"? sp:" "?){return ex})+"\n" {return {"line":_.flatten(e)}}

network =
	i:"=>"? " "? id:name args:args?" "? o:"=>"? "\n" lines:line* "\n" {return {"name": id, "in":(i == "=>"), "out": (o == "=>"), "const": args, "lines": lines}}
