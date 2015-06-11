`perf-focus` is an in-progress tool that can you be used with `perf
script` to answer queries about runtimes. Note that this tool is under
heavy development and you should not expect stable usage at this
point.

### Matcher

When you use the tool, you must specify a matcher. The tool will
filter all the samples reported by perf and tell you how many samples
matched the matcher critera and how many did not. Here is a simple
example:

```
// Two equivalent invocations:
> perf script | perf-focus '{^middle::traits}, ..{^je_}'
> perf script | perf-focus '{^middle::traits}..{^je_}'
```

Reports what percentage of samples were taken when some function whose
name begins with `middle::traits` was on the stack and it invoked
(transitively) a function whose name began with `je_`. In the query
syntax, `{<regex>}` matches a single function whose name is given by
the embedded regular expression. The `,` operator first matches The
`..M` prefix skips over any number of frames before matching `M`.  It
can also be used as a binary operator, so that `M..N` is equivalent to
`M,..N`.

```
> perf script | perf-focus '{^a$},{^b$}'
```

Reports how often a function named `a` directly called a function
named `b`.

```
> perf script | perf-focus '{^a$},!..{^b$}'
```

Reports how often a function named `a` was found on the stack
*without* having (transitively) called a function named `b`.

### Graphs

You can also generate call graphs by passing one of the following
options:

```
--graph
--graph-callers
--graph-callees
```

The first option will generate call graphs including all the frames in
every matching sample. The `--graph-callers` option geneates a call
graph including only those frames that invoked the matched
code. `--graph-callees` only includes those frame sthat were called by
the matched code.

In the graph, each node and edge is labeled with a percentage,
indicating the percentage of samples in which it appeared. This
percentage is **always** an absolute percentage across all samples in
the run (it is not a percentage of the matching samples, in
particular).

By default, the graph includes the top 22 most significant functions
(and edges between them). You can include more or less by passing
`--threshold N` (to include the top N functions).

You can use `--rename <regex> <match>` to munge the names of functions
that appear in the graph. This can be useful for stripping parts
of the fn name, or coallescing functions:

```
// convert things like `middle::traits::select::foo::bar` to
// `middle::traits::select`:
--rename '(^middle::traits::[a-zA-Z0-9_]+)::.*' '$1'

// strip the last `...::XXX` suffix:
--rename '::[a-zA-Z0-9_]+$' ''
```

### Histograms

Instead of a graph, you can use the histogram options to just dump out the most common
functions and the percentage of samples in which they appeared (as above, all percentages
are absolute):

```
--hist
--hist-callers
--hist-callees
```

