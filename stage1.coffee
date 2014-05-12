PEG = require 'pegjs'
fs = require 'fs'
util = require 'util'
_ = require 'underscore'
coffee = require 'pegjs-coffee-plugin'
grammar = fs.readFileSync './rat.pegc', {encoding:"utf8"}
source = fs.readFileSync './temps.rat', {encoding:"utf8"}

p = PEG.buildParser grammar, plugins: [coffee]
v = _.compact p.parse source
i = if process.argv.length > 2 then parseInt(process.argv[2]) else 0
b = v[i]

uidgen = (a) ->
	("00"+a.y).slice(-3) + ("00"+a.x).slice(-3)

getEdgesTo = (n, g) ->
	out = []
	ny = n.y-1
	nx = n.x - g.filter((z) ->(z.y == n.y) and (z.x <= n.x) and (z.modif=="^")).length
	h = g.filter((z) -> (z["x"] == nx) && (z["y"] == ny))[0]
	if h != undefined
		o = [uidgen(h), uidgen(n)]
	if n.proc == "%"
		g.filter((y) -> y.x >= n.x & y.y == ny).forEach (e) ->
			out.push([uidgen(e), uidgen(n)])
	else if ny != 0 and n.refs[0] == undefined
		out.push(o)
	n.refs.forEach (r, i) ->
		if r[0] == "x"
			out.push [uidgen(h), uidgen(g.filter((x)-> x.refs.filter((y) -> ("o"+r[1]) == y).length)[0])]
		else if r[0] == "o" and i > 0
			out.push o
		else
			console.error([n, h, [nx,ny]])
	out

outText = v.map((b) ->
	exprs = _.flatten(b["lines"])
	nodes = exprs.map (p) ->
		label = if p.args.length == 0 then p.proc else if p.args.length == 1 then p.proc + "(" + p.args[0] + ")" else p.proc + "(" + (JSON.stringify p.args) + ")"
		[uidgen(p), {pname: p.proc, label: label, args:p.args}]
	
	edges = exprs.map((d)->getEdgesTo(d, exprs)).filter((x) -> x?).reduce(((x,y) -> x.concat(y)), []).filter((x) -> x != undefined)

	dropMe = nodes.filter((x) -> x[1].pname == "").filter((x) -> x?).map((x) -> x[0])
	edges = edges.filter((y) -> ((dropMe.lastIndexOf(y[0]) < 0) and (dropMe.lastIndexOf(y[1]) < 0)))
	nodes = nodes.filter((x) -> _.flatten(edges).lastIndexOf(x[0]) > -1)

	{"edges": edges, "nodes": nodes, "name":b.name, "inrx":b.in, "outtx":b.out, "consts":b.const, "args": b.args})

console.log JSON.stringify(outText, null, 2)
