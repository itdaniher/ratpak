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
	/ " " r:ref " "? {return r}
	/ "."r:$("-"? [0-9]+) {return parseInt(r)}

args = " "? "(" v:$value ")" " "? { return v}

modif = " "? op:[v^] " " {return op}

proc =
	" "? "(" " "? p:proc " "? ")" " "? {return p}
	/ n: name a: args { return {"proc": n, "args": a, "pos":{"x":line()-1, "y":column()-1}} }
	/ n: name r: ref+ {return {"proc":n, "refs": r, "pos":{"x":line()-1, "y":column()-1}} }
	/ n: name {return {"proc": n, "pos":{"x":line()-1, "y":column()-1} }}

expr =
	" "? "(" " "? e:expr " "? ")" " "? {return e}
	/ proc
	/ d: modif rp: proc {return _.extend(rp, {"modif": d})}
	/ "(" " "? d: modif  rp:(proc " "?)+ " "? ")" {
		out = [];
		rp.map(function(x) {x[0]["modif"] = d; out.push(x[0]);});
		delete(out[0]["modif"]);
		return out
	}

line =
	"\t" " "* e:((ex:expr " "?){return ex})+ "\n" {return _.flatten(e)}

network =
	i:"=>"? " "? id:name args:args?" "? o:"=>"? "\n" lines:line* "\n" {return {"name": id, "in":(i == "=>"), "out": (o == "=>"), "const": args, "lines": lines}}
