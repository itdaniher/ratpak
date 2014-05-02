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

exprs = _.flatten(b["lines"])

nodes = exprs.map (p) ->
	label = if p.args.length == 0 then p.proc else p.proc + "(" + (JSON.stringify p.args) + ")"
	[uidgen(p), {pname: p.proc, label: label, args:p.args}]

getEdgesTo = (n, g) ->
	out = []
	if n.y != 1 and n.refs[0] == undefined
		out.push([uidgen(_.findWhere(g, {"x": n.x-(n.modif=="^"), "y": n.y-1})), uidgen(n)])
	n.refs.forEach (r) ->
		if n.refs[0] < 0
			out.push([uidgen(_.findWhere(g, {"x": n.x-(n.modif=="^"), "y": n.y-1})), uidgen(g.filter((x)-> x.refs.filter((y) -> y==-r).length)[0])])
		else
			out.push([(uidgen(g.filter((z) -> (z["x"] == (n.x-(n.modif=="^"))) && (z["y"] == n.y-1))[0])), uidgen(n)])
	out[0]

edges = [getEdgesTo d, exprs for d in exprs][0].filter((x) -> x?)

dropMe = nodes.filter((x) -> x[1].pname == "\"").filter((x) -> x?).map((x) -> x[0])
edges = edges.filter((y) -> ((dropMe.lastIndexOf(y[0]) < 0) and (dropMe.lastIndexOf(y[1]) < 0)))
nodes = nodes.filter((x) -> _.flatten(edges).lastIndexOf(x[0]) > -1)

outText = JSON.stringify({"edges": edges, "nodes": nodes, "name":b.name, "inrx":b.in, "outtx":b.out, "consts":b.const}, null, 2)
console.log outText
