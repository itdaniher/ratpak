PEG = require 'pegjs'
fs = require 'fs'
util = require 'util'
_ = require 'underscore'
msgpack = require 'msgpack'
coffee = require 'pegjs-coffee-plugin'

grammar = fs.readFileSync './rat.pegc', {encoding:"utf8"}
source = fs.readFileSync './temps.rat', {encoding:"utf8"}

p = PEG.buildParser grammar, plugins: [coffee]
v = _.compact p.parse source
b = v[0]

console.log JSON.stringify b, null, 1

pname = (a) ->
	a.proc + ("00"+a.pos.y).slice(-3) + ("00"+a.pos.x).slice(-3)

nodes = []

_.flatten(b["lines"]).map((p) ->
	if p.refs == undefined or p.refs[0] >= 0
		if p.proc != "\""
			label = if (p.args) == undefined then p.proc else p.proc + "(" + (JSON.stringify p.args) + ")"
			nodes.push [pname(p), {proc: p.proc, label: label, mass:label.length, args:p.args}]
)

edges = []

if b.in
	nodes.push ["in000000", {"label": "in", "mass":1}]
	b["lines"].unshift([{"proc": "in", "pos":{"x":0, "y":0}}])

b["lines"].forEach((e, i, l) ->
	e.forEach (ee, ii, ll) ->
		if i != 0
			ai = ii
			ai -= ll.slice(1, ii+1).filter((p) -> p.modif == "^").length # drop forked procs from the running
			ai += l[i-1].slice(0, ii+1).filter((p) -> if p.refs then p.refs[0] < 0 else false).length # drop refs on last line
			upper = l[i-1][ai]
			if ee.modif == "v"
				edges.push [pname(ee), pname(l[i+1][ii-1])]
			else if ee.modif == "/"
				[1..l[i-1].length-ii-1].forEach (x) ->
					edges.push [pname(l[i-1][ai+x]), pname(ee)]
			if ee.refs == undefined or ee.refs.filter((p) -> p > 0).length
				edges.push [pname(upper), pname(ee)]
			else if ee.refs.filter((p) -> p < 0).length
				ee.refs.forEach (eee) ->
					back = (p for p in l[i+eee] when p.refs and -1*eee in p.refs)[0]
					edges.push [pname(upper), pname(back), {label: eee}]
		)

if b.out
	nodes.push ["out", {"label": "out", "mass":1}]
	edges.push [edges[edges.length-1][1], "out"]

edges2 = []

for edge in edges
	if edge[0][0] == "\"" or edge[1][0] == "\""
		if edge[0][0] == "\""
			for source in (e for e in edges when e[1] == edge[0])
				edges2.push [source[0], edge[1]]
	else
		edges2.push edge

for node in nodes
	ict = 0
	oct = 0
	for e in edges
		if e[0] == node[0]
			oct += 1
		if e[1] == node[0]
			ict += 1
	node[1]["ict"] = ict
	node[1]["oct"] = oct

fs.writeFileSync "./temps.msgpack", msgpack.pack {"edges": edges, "nodes": nodes}

fs.writeFileSync "./springy/test.json", JSON.stringify({"edges": edges2, "nodes": nodes}, null, 2), {encoding: "utf8"}
