PEG = require 'pegjs'
fs = require 'fs'
util = require 'util'
_ = require 'underscore'

grammar = fs.readFileSync './rat.pegjs', {encoding:"utf8"}
source = fs.readFileSync './temps.rat', {encoding:"utf8"}

p = PEG.buildParser grammar
v = _.compact p.parse source
b = v[2]

console.log JSON.stringify b, null, 1

pname = (a) ->
	a.proc + ("00"+a.pos.x).slice(-3) + ("00"+a.pos.y).slice(-3)

nodes = []

_.flatten(b["lines"]).map((p) ->
	if p.refs == undefined or p.refs[0] > 0
		label = if (p.args) == undefined then p.proc else p.proc + "(" + p.args + ")"
		nodes.push [pname(p), {"label": label, "mass":p.length}]
)

edges = []

b["lines"].forEach((e, i, l) ->
	e.forEach (ee, ii, ll) ->
		if i != 0
			ai = ii # base addr
			ai -= ll.slice(1, ii+1).filter((p) -> p.modif == "^").length # drop forked procs from the running
			ai += l[i-1].slice(0, ii+1).filter((p) -> if p.refs then p.refs[0] < 0 else false).length # drop refs on last line
			upper = l[i-1][ai]
			if ee.modif == "v"
				edges.push [pname(ee), pname(l[i+1][ii-1])]
			if ee.refs == undefined or ee.refs.filter((p) -> p>0).length
				edges.push [pname(upper), pname(ee)]
			else
				ee.refs.forEach (eee, iii, lll) ->
					back = (p for p in l[i+eee] when p.refs and -1*eee in p.refs)[0]
					edges.push [pname(upper), pname(back), {label: eee}]
		)

if b.out
	nodes.push ["out", {"label": "out", "mass":1}]
	edges.push [edges[edges.length-1][1], "out"]

if b.in
	nodes.push ["in", {"label": "in", "mass":1}]
	edges.push ["in", nodes[0][0]]

fs.writeFileSync "./springy/test.json", JSON.stringify({"edges": edges, "nodes": nodes}, null, 2), {encoding: "utf8"}
