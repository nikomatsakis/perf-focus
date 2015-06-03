`perf-focus` is an in-progress tool that can you be used with `perf
script` to answer queries about runtimes. In lieu of real
documentation, here are some examples of how you can use it:

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
