# temper ctl playground

Kind: reference

The playground: start it, or end it and set up temper.

## playground start

```
playground start [--key KEY]
Start the playground; run it again to finish what is not done
Prints: `running`
--json: the record playground-start.
Key: --key KEY, optional.
It returns once the playground runs. A failed start names its step,
the fakes, the account, the adoption or the forge, and the same
call again finishes it. `refused: not offered` (exit 2): this
deployment has no playground.
```

## playground end

```
playground end [--key KEY]
End the playground and set up temper
Prints: `reset`, or `refused: not offered` (exit 2)
--json: the record playground-end.
Key: --key KEY, optional.
```
