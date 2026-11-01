# temper ctl playground

Kind: reference

The playground: start it, or reset it.

## playground start

```
playground start [--key KEY]
Start the playground; run it again to finish what is not done
Prints: `running`
--json: the record demo-start.
Key: --key KEY, optional.
It returns once the playground runs. A failed start names its step,
the fakes, the account, the adoption or the forge, and the same
call again finishes it. `refused: not offered` (exit 2): this
deployment has no playground.
```

## playground reset

```
playground reset [--key KEY]
Reset the playground: its project, forge and account are removed, and the welcome opens at its Forge step
Prints: `reset`
--json: the record playground-reset.
Key: --key KEY, optional.
It returns once the playground is gone. `refused: not offered`
(exit 2): no playground runs.
```
