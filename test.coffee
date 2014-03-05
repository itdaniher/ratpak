PEG = require 'pegjs'
fs = require 'fs'
util = require 'util'
_ = require 'underscore'

grammar = fs.readFileSync './rat.pegjs', {encoding:"utf8"}
source = fs.readFileSync './temps.rat', {encoding:"utf8"}

p = PEG.buildParser grammar
v = _.compact p.parse source
b = v[0]

pname = (a) ->
	a.proc + ("00"+a.pos.x).slice(-3) + ("00"+a.pos.y).slice(-3)

nodes = []

_.flatten(b["lines"]).map((p) ->
	if not p.refs or p.refs[0] > 0
		label = if (p.args) == undefined then p.proc else p.proc + "(" + p.args + ")"
		nodes.push [pname(p), {"label": label, "mass":p.length}]
)

edges = []

b["lines"].forEach((e, i, l) ->
	e.forEach (ee, ii, ll) ->
		if i != 0
			ai = ii - ll.slice(0,ii+1).filter((p) -> p.dyad == "^").length + l[i-1].slice(0, ii+1).filter((p) -> if p.refs then p.refs[0] < 0 else false).length
			if ee.dyad == "^"
				upper = l[i-1][ai]
			else if ee.dyad == "v"
				upper = l[i-1][ai]
				edges.push [pname(ee), pname(l[i+1][ii-1])]
			else
				upper = l[i-1][ai]
			if ee.refs == undefined or ee.refs.filter((p) -> p>0).length
				edges.push [pname(upper), pname(ee)]
			else
				ee.refs.forEach (eee, iii, lll) ->
					edges.push [pname(upper), (pname(p) for p in l[i+eee] when p.refs and -1*eee in p.refs)[0], {label: eee}]
		)

if b.out
	nodes.push ["out", {"label": "out"}]
	edges.push [edges[edges.length-1][1], "out"]

if b.in
	nodes.push ["in", {"label": "in"}]
	edges.push ["in", nodes[0][0]]

fs.writeFileSync "./springy/test.json", JSON.stringify({"edges": edges, "nodes": nodes}, null, 2), {encoding: "utf8"}
